use ::futures::{ future, Future, Stream };
use ::ResultExt;

/// Computes which old mods to uninstall and which new mods to install based on the given reqs.
/// Asks the user for confirmation, then applies the diff.
///
/// Returns true if the diff was successfully applied or empty.
pub fn compute_and_apply_diff<'a>(
	local_api: &'a ::factorio_mods_local::API,
	web_api: &'a ::factorio_mods_web::API,
	reqs: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
) -> impl Future<Item = (bool, ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>), Error = ::Error> + 'a {
	::async_block! {
		let (solution, reqs) = ::await!(solve(web_api, local_api.game_version(), reqs))?;
		let solution = solution.ok_or_else(|| "No solution found.")?;
		let (to_uninstall, to_install) = match compute_diff(solution, local_api)? {
			Some(diff) => diff,
			None => return Ok((false, reqs)),
		};

		if to_uninstall.is_empty() && to_install.is_empty() {
			return Ok((true, reqs));
		}

		let user_credentials = ::await!(::util::ensure_user_credentials(local_api, web_api))?;

		println!();

		Ok(::await!(with_multi_progress("Applying solution...", move |apply_diff_multi_progress| ::async_block! {
			let mods_directory = local_api.mods_directory();
			let mods_directory_canonicalized = mods_directory.canonicalize().chain_err(|| format!("Could not canonicalize {}", mods_directory.display()))?;

			let uninstall_futures = {
				let apply_diff_multi_progress = apply_diff_multi_progress.clone();
				to_uninstall.into_iter().map(move |installed_mod| future::Either::A(with_progress_bar(
					&apply_diff_multi_progress,
					&format!("    Removing {} {}", installed_mod.info().name(), installed_mod.info().version()),
					|uninstall_progress_bar| ::async_block! {
						let result = match installed_mod.mod_type() {
							::factorio_mods_local::InstalledModType::Zipped => {
								let path = installed_mod.path();
								uninstall_progress_bar.set_message(&format!("removing file {}", path.display()));
								::std::fs::remove_file(path).chain_err(|| format!("Could not remove file {}", path.display()))
							},

							::factorio_mods_local::InstalledModType::Unpacked => {
								let path = installed_mod.path();
								uninstall_progress_bar.set_message(&format!("removing directory {}", path.display()));
								::std::fs::remove_dir_all(path).chain_err(|| format!("Could not remove directory {}", path.display()))
							},
						};

						match result {
							Ok(()) => Ok((uninstall_progress_bar, ())),
							Err(err) => Err((uninstall_progress_bar, err)),
						}
					})))
			};

			let install_futures =
				to_install
				.into_iter()
				.map(move |release| {
					let chunk_stream = web_api.download(&release, &user_credentials);

					let filename = mods_directory.join(release.filename());
					let displayable_filename = filename.display().to_string();

					let result = do catch {
						let mut download_filename: ::std::ffi::OsString =
							filename.file_name()
							.ok_or_else(|| format!("Could not parse filename {}", displayable_filename))?
							.into();

						download_filename.push(".new");
						let download_filename = filename.with_file_name(download_filename);
						let download_displayable_filename = download_filename.display().to_string();

						{
							let parent = download_filename.parent().ok_or_else(|| format!("Filename {} is malformed", download_displayable_filename))?;
							let parent_canonicalized = parent.canonicalize().chain_err(|| format!("Filename {} is malformed", download_displayable_filename))?;
							if parent_canonicalized != mods_directory_canonicalized {
								Err(format!("Filename {} is malformed", download_displayable_filename))?;
							}
						}

						let mut file = ::std::fs::OpenOptions::new();
						let file = file.create(true).truncate(true).write(true);
						let file = file.open(&download_filename).chain_err(|| format!("Could not open {} for writing", download_displayable_filename))?;
						let writer = ::std::io::BufWriter::new(file);

						Ok((download_filename, download_displayable_filename, writer))
					};

					future::Either::B(with_progress_bar(
						&apply_diff_multi_progress,
						&format!("    Installing {} {}", release.info_json().name(), release.info_json().version()),
						|install_progress_bar| ::async_block! {
							let (download_filename, download_displayable_filename, mut writer) = match result {
								Ok(value) => value,
								Err(err) => return Err((install_progress_bar, err)),
							};

							install_progress_bar.set_message(&format!("downloading to {} ...", download_displayable_filename));

							let mut chunk_stream = chunk_stream;
							let result = loop {
								match ::await!(chunk_stream.into_future()) {
									Ok((Some(chunk), rest)) => {
										if let Err(err) = ::std::io::Write::write_all(&mut writer, &chunk) {
											break Err(err).chain_err(|| format!("Could not write to file {}", download_displayable_filename));
										}

										chunk_stream = rest;
									},

									Ok((None, _)) => {
										install_progress_bar.set_message(&format!("renaming {} to {} ...", download_displayable_filename, displayable_filename));
										break ::std::fs::rename(&download_filename, &filename)
										.chain_err(|| format!("Could not rename {} to {}", download_displayable_filename, displayable_filename));
									},

									Err((err, _)) =>
										break Err(err).chain_err(|| format!("Could not download release {} {}", release.info_json().name(), release.version())),
								}
							};
							match result {
								Ok(()) => Ok((install_progress_bar, ())),
								Err(err) => Err((install_progress_bar, err)),
							}
						}))
				});

			let _: Vec<()> = ::await!(future::join_all(uninstall_futures.chain(install_futures)))?;

			Ok::<_, ::Error>((true, reqs))
		}))?)
	}
}

struct Cache {
	graph: ::petgraph::Graph<Installable, Relation>,
	already_fetching: ::std::collections::HashSet<::factorio_mods_common::ModName>,
}

#[derive(Clone, Debug)]
enum Installable {
	Base(::factorio_mods_common::ModName, ::factorio_mods_common::ReleaseVersion),
	Mod(::factorio_mods_web::ModRelease),
}

impl Installable {
	fn name(&self) -> &::factorio_mods_common::ModName {
		match *self {
			Installable::Base(ref name, _) => name,
			Installable::Mod(ref release) => release.info_json().name(),
		}
	}

	fn version(&self) -> &::factorio_mods_common::ReleaseVersion {
		match *self {
			Installable::Base(_, ref version) => version,
			Installable::Mod(ref release) => release.version(),
		}
	}

	fn dependencies(&self) -> &[::factorio_mods_common::Dependency] {
		match *self {
			Installable::Base(..) => &[],
			Installable::Mod(ref release) => release.info_json().dependencies(),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Relation {
	Requires,
	Conflicts,
}

fn solve<'a>(
	api: &'a ::factorio_mods_web::API,
	game_version: &'a ::factorio_mods_common::ReleaseVersion,
	reqs: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
) -> impl Future<Item = (
	Option<::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_web::ModRelease>>,
	::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
), Error = ::Error> + 'a {
	::async_block! {
		let cache = ::futures_mutex::FutMutex::new(Cache {
			graph: Default::default(),
			already_fetching: Default::default(),
		});

		let (reqs, cache) = ::await!(with_multi_progress("Fetching releases...", move |fetch_mods_multi_progress| ::async_block! {
			let cache = ::await!(add_installable(cache, Installable::Base(::factorio_mods_common::ModName::new("base".to_string()), game_version.clone())))?;
			let futures: Vec<_> =
				reqs.keys()
				.map(|name| add_mod(api, game_version, cache.clone(), name.clone(), fetch_mods_multi_progress.clone()))
				.collect();

			let _: Vec<()> = ::await!(future::join_all(futures))?;

			Ok::<_, ::Error>((reqs, cache))
		}))?;

		#[allow(unreachable_code, unreachable_patterns)]
		let Ok(mut guard) = ::await!(lock(cache));

		let graph = ::std::mem::replace(&mut (*guard).graph, Default::default());
		let solution = compute_solution(graph, &reqs)?;
		Ok((solution, reqs))
	}
}

fn compute_diff(
	mut solution: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_web::ModRelease>,
	local_api: &::factorio_mods_local::API,
) -> ::Result<Option<(Vec<::factorio_mods_local::InstalledMod>, Vec<::factorio_mods_web::ModRelease>)>> {
	let all_installed_mods: ::Result<::multimap::MultiMap<_, _>> =
		local_api.installed_mods().chain_err(|| "Could not enumerate installed mods")?
		.map(|mod_|
			mod_
			.map(|mod_| (mod_.info().name().clone(), mod_))
			.chain_err(|| "Could not process an installed mod"))
		.collect();

	let all_installed_mods = all_installed_mods.chain_err(|| "Could not enumerate installed mods")?;

	let mut to_uninstall = vec![];
	let mut to_install = ::std::collections::HashMap::new();

	for (name, installed_mods) in all_installed_mods {
		match solution.remove(&name) {
			Some(release) => {
				let mut already_installed = false;

				for installed_mod in installed_mods {
					if release.version() == installed_mod.info().version() {
						already_installed = true;
					}
					else {
						to_uninstall.push(installed_mod);
					}
				}

				if !already_installed {
					to_install.insert(name.clone(), release);
				}
			},

			None =>
				to_uninstall.extend(installed_mods),
		}
	}

	to_install.extend(solution);

	{
		let to_upgrade =
			::itertools::Itertools::sorted_by(
				to_uninstall.iter().filter_map(|installed_mod|
					to_install.get(installed_mod.info().name())
					.map(|release| (installed_mod, release))),
				|&(installed_mod1, release1), &(installed_mod2, release2)|
					installed_mod1.info().name().cmp(installed_mod2.info().name())
					.then_with(|| installed_mod1.info().version().cmp(installed_mod2.info().version()))
					.then_with(|| release1.info_json().name().cmp(release2.info_json().name()))
					.then_with(|| release1.version().cmp(release2.version())));

		if !to_upgrade.is_empty() {
			println!();
			println!("The following mods will be upgraded:");
			for (installed_mod, release) in to_upgrade {
				println!("    {} {} -> {}", installed_mod.info().name(), installed_mod.info().version(), release.version());
			}
		}
	}

	to_uninstall.sort_by(|installed_mod1, installed_mod2|
		installed_mod1.info().name().cmp(installed_mod2.info().name())
		.then_with(|| installed_mod1.info().version().cmp(installed_mod2.info().version())));

	let to_install =
		::itertools::Itertools::sorted_by(to_install.into_iter().map(|(_, release)| release), |release1, release2|
			release1.info_json().name().cmp(release2.info_json().name())
			.then_with(|| release1.version().cmp(release2.version())));

	if !to_uninstall.is_empty() {
		println!();
		println!("The following mods will be removed:");
		for installed_mod in &to_uninstall {
			println!("    {} {}", installed_mod.info().name(), installed_mod.info().version());
		}
	}

	if !to_install.is_empty() {
		println!();
		println!("The following new mods will be installed:");
		for release in &to_install {
			println!("    {} {}", release.info_json().name(), release.version());
		}
	}

	println!();

	if to_uninstall.is_empty() && to_install.is_empty() {
		println!("Nothing to do.");
	}
	else if !::util::prompt_continue()? {
		return Ok(None);
	}

	Ok(Some((to_uninstall, to_install)))
}

fn add_mod<'a>(
	api: &'a ::factorio_mods_web::API,
	game_version: &'a ::factorio_mods_common::ReleaseVersion,
	cache: ::futures_mutex::FutMutex<Cache>,
	name: ::factorio_mods_common::ModName,
	fetch_mods_multi_progress: ::std::sync::Arc<::indicatif::MultiProgress>,
) -> Box<Future<Item = (), Error = ::Error> + 'a> {
	Box::new(::async_block! {
		#[allow(unreachable_code, unreachable_patterns)]
		let Ok(mut guard) = ::await!(lock(cache));

		{
			let cache = &mut *guard;

			if !cache.already_fetching.insert(name.clone()) {
				return Ok(());
			}
		}

		let cache = guard.unlock();

		::await!(with_progress_bar(&fetch_mods_multi_progress.clone(), &format!("    {}", name), move |progress_bar| ::async_block! {
			progress_bar.set_message("fetching...");

			let result = match ::await!(api.get(&name)) {
				Ok(mod_) => {
					let add_releases_and_deps_futures: Vec<_> =
						mod_.releases().into_iter()
						.flat_map(|release| if release.factorio_version().matches(game_version) {
							let mut futures: Vec<_> =
								release.info_json().dependencies()
								.into_iter()
								.filter_map(|dep| if dep.required() && &**dep.name() != "base" {
									Some(add_mod(api, game_version, cache.clone(), dep.name().clone(), fetch_mods_multi_progress.clone()))
								}
								else {
									None
								})
								.collect();

							futures.push(Box::new(add_installable(cache.clone(), Installable::Mod(release.clone())).map(|_| ())));

							futures
						}
						else {
							vec![]
						})
						.collect(); // Force eager evaluation to remove dependency on lifetime of `mod_`

					::await!(future::join_all(add_releases_and_deps_futures).map(|_: Vec<()>| ()))
				},

				Err(err) => match *err.kind() {
					// Don't fail the whole process due to non-existent deps. Releases with unmet deps will be handled when computing the solution.
					::factorio_mods_web::ErrorKind::StatusCode(_, ::factorio_mods_web::reqwest::StatusCode::NotFound) => Ok(()),

					_ => Err(err).chain_err(|| format!("Could not get mod info for {}", name)),
				},
			};

			match result {
				Ok(()) => Ok((progress_bar, ())),
				Err(err) => Err((progress_bar, err)),
			}
		}))?;

		Ok(())
	})
}

fn add_installable(
	cache: ::futures_mutex::FutMutex<Cache>,
	installable: Installable,
) -> impl Future<Item = ::futures_mutex::FutMutex<Cache>, Error = ::Error> + 'static {
	::async_block! {
		#[allow(unreachable_code, unreachable_patterns)]
		let Ok(mut guard) = ::await!(lock(cache));

		{
			let cache = &mut *guard;
			cache.graph.add_node(installable);
		}

		Ok(guard.unlock())
	}
}

fn compute_solution(
	mut graph: ::petgraph::Graph<Installable, Relation>,
	reqs: &::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
) -> ::Result<Option<::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_web::ModRelease>>> {
	println!();
	println!("Computing solution...");

	let mut edges_to_add = vec![];

	for node_index1 in graph.node_indices() {
		let installable1 = &graph[node_index1];

		for node_index2 in graph.node_indices() {
			if node_index1 == node_index2 {
				continue;
			}

			let installable2 = &graph[node_index2];

			if installable1.name() == installable2.name() {
				edges_to_add.push((node_index1, node_index2, Relation::Conflicts));
			}
			else {
				let mut requires = false;
				let mut conflicts = false;

				for dep in installable1.dependencies() {
					if dep.name() != installable2.name() {
						continue;
					}

					match (dep.required(), dep.version().matches(installable2.version())) {
						(true, true) => requires = true,
						(false, false) => conflicts = true,
						_ => continue,
					}
				}

				match (requires, conflicts) {
					(true, true) => bail!("{} {} both requires and conflicts with {} {}", installable1.name(), installable1.version(), installable2.name(), installable2.version()),
					(true, false) => edges_to_add.push((node_index1, node_index2, Relation::Requires)),
					(false, true) => edges_to_add.push((node_index1, node_index2, Relation::Conflicts)),
					(false, false) => (),
				}
			}
		}
	}

	for edge_to_add in edges_to_add {
		assert!(graph.find_edge(edge_to_add.0, edge_to_add.1).is_none());

		graph.add_edge(edge_to_add.0, edge_to_add.1, edge_to_add.2);
	}

	loop {
		let mut node_indices_to_remove = ::std::collections::HashSet::new();

		{
			let name_to_node_indices: ::multimap::MultiMap<_, _> = graph.node_indices().map(|node_index| {
				let installable = &graph[node_index];
				(installable.name(), node_index)
			}).collect();

			for name in reqs.keys() {
				match name_to_node_indices.get_vec(name) {
					Some(node_indices) if !node_indices.is_empty() => (),
					_ => bail!("No valid installable releases found for {}", name),
				}
			}

			node_indices_to_remove.extend(graph.node_indices().filter(|&node_index| {
				let installable = &graph[node_index];

				let keep = match reqs.get(installable.name()) {
					// Required installable
					Some(req) => req.matches(installable.version()),

					// Required by another installable
					None => graph.edges_directed(node_index, ::petgraph::Direction::Incoming).any(|edge|
						if let Relation::Requires = *edge.weight() {
							true
						}
						else {
							false
						}),
				};

				// All required dependencies satisfied
				let keep = keep &&
					installable.dependencies().into_iter()
					.filter(|dep| dep.required()).all(|dep|
						name_to_node_indices.get_vec(dep.name()).unwrap().into_iter()
						.any(|&dep_node_index| dep.version().matches(graph[dep_node_index].version())));

				!keep
			}));

			if node_indices_to_remove.is_empty() {
				for (_, node_indices) in name_to_node_indices.iter_all() {
					for &node_index1 in node_indices {
						let installable1 = &graph[node_index1];

						let neighbors1: ::std::collections::HashSet<_> =
							graph.edges_directed(node_index1, ::petgraph::Direction::Incoming)
							.map(|edge| (::petgraph::Direction::Incoming, edge.weight(), ::petgraph::visit::EdgeRef::source(&edge)))
							.chain(
								graph.edges(node_index1)
								.map(|edge| (::petgraph::Direction::Outgoing, edge.weight(), ::petgraph::visit::EdgeRef::target(&edge))))
							.filter(|&(_, _, neighbor_node_index)| graph[neighbor_node_index].name() != installable1.name())
							.collect();

						for &node_index2 in node_indices {
							if node_index2 > node_index1 {
								let installable2 = &graph[node_index2];

								let neighbors2: ::std::collections::HashSet<_> =
									graph.edges_directed(node_index2, ::petgraph::Direction::Incoming)
									.map(|edge| (::petgraph::Direction::Incoming, edge.weight(), ::petgraph::visit::EdgeRef::source(&edge)))
									.chain(
										graph.edges(node_index2)
										.map(|edge| (::petgraph::Direction::Outgoing, edge.weight(), ::petgraph::visit::EdgeRef::target(&edge))))
									.filter(|&(_, _, neighbor_node_index)| graph[neighbor_node_index].name() != installable2.name())
									.collect();

								if neighbors1 == neighbors2 {
									// Two installables with identical requirements and conflicts. Remove the one with the lower version.
									if installable1.version() < installable2.version() {
										node_indices_to_remove.insert(node_index1);
									}
									else {
										node_indices_to_remove.insert(node_index2);
									}
								}
							}
						}
					}
				}
			}

			if node_indices_to_remove.is_empty() {
				for req in reqs.keys() {
					let node_indices = name_to_node_indices.get_vec(req).unwrap();

					let mut common_conflicts = None;

					for &node_index in node_indices {
						let conflicts: ::std::collections::HashSet<_> =
							graph.edges(node_index)
							.filter_map(|edge|
								if let Relation::Conflicts = *edge.weight() {
									Some(::petgraph::visit::EdgeRef::target(&edge))
								}
								else {
									None
								})
							.collect();

						common_conflicts = if let Some(existing) = common_conflicts {
							Some(&existing & &conflicts)
						}
						else {
							Some(conflicts)
						};
					}

					if let Some(common_conflicts) = common_conflicts {
						node_indices_to_remove.extend(common_conflicts);
					}
				}
			}
		}

		if node_indices_to_remove.is_empty() {
			break;
		}
		else {
			let node_indices_to_remove = ::itertools::Itertools::sorted_by(node_indices_to_remove.into_iter(), |i1, i2| i1.cmp(i2).reverse());

			for node_index in node_indices_to_remove {
				graph.remove_node(node_index);
			}
		}
	}

	let possibilities: Vec<_> = {
		let name_to_installables: ::multimap::MultiMap<_, _> =
			graph.into_nodes_edges().0.into_iter().map(|node| {
				let installable = node.weight;
				(installable.name().clone(), Some(installable))
			}).collect();

		name_to_installables.into_iter().map(|(name, mut installables)| {
			if &*name != "base" && !reqs.contains_key(&name) {
				installables.insert(0, None);
			}

			installables
		}).collect()
	};

	let possibilities: Vec<Vec<_>> = possibilities.iter().map(|p| p.iter().map(Option::as_ref).collect()).collect();
	let possibilities: Vec<_> = possibilities.iter().map(AsRef::as_ref).collect();
	let mut permutater = Permutater::new(&possibilities[..]);

	let mut values = vec![None; possibilities.len()];

	let mut best_solution = None;

	while permutater.next(&mut values) {
		let solution = values.iter().filter_map(|installable| installable.map(|installable| (installable.name(), installable))).collect();

		if is_valid(&solution) {
			best_solution = ::std::cmp::max(best_solution, Some(Solution(solution)));
		}
	}

	Ok(best_solution.map(|best_solution| best_solution.0.into_iter().filter_map(|(name, installable)| {
		if let Installable::Mod(ref release) = *installable {
			Some((name.clone(), release.clone()))
		}
		else {
			None
		}
	}).collect()))
}

fn is_valid(solution: &::std::collections::HashMap<&::factorio_mods_common::ModName, &Installable>) -> bool {
	for installable in solution.values() {
		for dep in installable.dependencies() {
			if let Some(installable) = solution.get(dep.name()) {
				if !dep.version().matches(installable.version()) {
					return false;
				}
			}
			else if dep.required() {
				return false;
			}
		}
	}

	true
}

#[derive(Debug)]
struct Solution<'a>(::std::collections::HashMap<&'a ::factorio_mods_common::ModName, &'a Installable>);

impl<'a> Ord for Solution<'a> {
	fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
		for (n1, i1) in &self.0 {
			if let Some(i2) = other.0.get(n1) {
				match i1.version().cmp(i2.version()) {
					::std::cmp::Ordering::Equal => (),
					o => return o,
				}
			}
		}

		self.0.len().cmp(&other.0.len()).reverse()
	}
}

impl<'a> PartialOrd for Solution<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> PartialEq for Solution<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == ::std::cmp::Ordering::Equal
	}
}

impl<'a> Eq for Solution<'a> {
}

struct Permutater<'a, T> where T: 'a {
	state: Vec<usize>,
	possibilities: &'a [&'a [T]],
	run_once: bool,
}

impl<'a, T> Permutater<'a, T> where T: Copy {
	fn new(possibilities: &'a [&'a [T]]) -> Permutater<'a, T> {
		Permutater {
			state: vec![0; possibilities.len()],
			possibilities,
			run_once: false,
		}
	}

	fn next(&mut self, values: &mut [T]) -> bool {
		assert_eq!(self.possibilities.len(), values.len());

		if self.advance(0) {
			for (value_index, &element_index) in self.state.iter().enumerate() {
				values[value_index] = self.possibilities[value_index][element_index];
			}

			true
		}
		else {
			false
		}
	}

	fn advance(&mut self, index: usize) -> bool {
		if index >= self.possibilities.len() {
			return false;
		}

		if self.run_once {
			if self.state[index] < self.possibilities[index].len() - 1 {
				self.state[index] += 1;
				true
			}
			else {
				self.state[index] = 0;
				self.advance(index + 1)
			}
		}
		else {
			self.run_once = true;
			true
		}
	}
}

fn lock<T>(mutex: ::futures_mutex::FutMutex<T>) -> impl Future<Item = ::futures_mutex::FutMutexAcquired<T>, Error = !> {
	mutex.lock().map_err(|()| unreachable!())
}

fn with_multi_progress<F, A>(
	title: &str,
	f: F,
) -> impl Future<Item = A::Item, Error = A::Error>
where
	F: FnOnce(::std::sync::Arc<::indicatif::MultiProgress>) -> A,
	A: Future,
{
	let multi_progress = ::std::sync::Arc::new(::indicatif::MultiProgress::new());

	// ProgressBars will be add()ed to the MultiProgress dynamically, but it needs to be join()ed immediately to be able to draw itself.
	// To prevent it from finishing before any ProgressBars are actually added to it, add one ProgressBar that doesn't finish
	// until signaled.
	let title_progress_bar = {
		let title_progress_bar = multi_progress.add(::indicatif::ProgressBar::new(1));
		title_progress_bar.set_style(::indicatif::ProgressStyle::default_bar().template("{prefix}"));
		title_progress_bar.set_prefix(title);
		title_progress_bar
	};

	::async_block! {
		let multi_progress_receiver = {
			let (multi_progress_sender, multi_progress_receiver) = ::futures::sync::oneshot::channel();
			let multi_progress = multi_progress.clone();

			::std::thread::spawn(move || {
				multi_progress.join().unwrap();
				multi_progress_sender.send(()).unwrap();
			});

			multi_progress_receiver
		};

		let result = ::await!(f(multi_progress));

		title_progress_bar.finish();

		::await!(multi_progress_receiver).unwrap();

		result
	}
}

fn with_progress_bar<F, A, T, E>(
	multi_progress: &::indicatif::MultiProgress,
	prefix: &str,
	f: F,
) -> impl Future<Item = T, Error = E>
where
	F: FnOnce(::indicatif::ProgressBar) -> A,
	A: Future<Item = (::indicatif::ProgressBar, T), Error = (::indicatif::ProgressBar, E)>,
{
	let progress_bar = multi_progress.add(::indicatif::ProgressBar::new(1));
	progress_bar.set_style(::indicatif::ProgressStyle::default_bar().template("{prefix} {wide_msg:>}"));
	progress_bar.set_prefix(prefix);

	::async_block! {
		match ::await!(f(progress_bar)) {
			Ok((progress_bar, value)) => {
				progress_bar.finish_with_message("done");
				Ok(value)
			},

			Err((progress_bar, err)) => {
				progress_bar.finish_with_message("failed");
				Err(err)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_permutater() {
		let possibilities = vec![vec![None, Some("a"), Some("b")], vec![None, Some("c")]];
		let possibilities: Vec<_> = possibilities.iter().map(AsRef::as_ref).collect();
		let mut values = vec![None; possibilities.len()];

		let mut permutater = Permutater::new(&possibilities);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [None, None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("a"), None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("b"), None]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [None, Some("c")]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("a"), Some("c")]);

		assert!(permutater.next(&mut values));
		assert_eq!(values, [Some("b"), Some("c")]);

		assert!(!permutater.next(&mut values));
	}
}
