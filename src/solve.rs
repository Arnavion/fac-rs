use ::futures::{ future, Async, Future, IntoFuture, Poll, Stream };
use ::ResultExt;

/// Computes which old mods to uninstall and which new mods to install based on the given reqs.
/// Asks the user for confirmation, then applies the diff.
///
/// Returns true if the diff was successfully applied or empty.
pub fn compute_and_apply_diff<'a>(
	local_api: &'a ::factorio_mods_local::API,
	web_api: &'a ::factorio_mods_web::API,
	reqs: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
) -> impl Future<Item = bool, Error = ::Error> + 'a {
	solve(web_api, local_api.game_version(), reqs)
	.and_then(|solution| solution.ok_or_else(|| "No solution found.".into()))
	.and_then(move |solution| compute_diff(solution, local_api))
	.and_then(move |diff| if let Some((to_uninstall, to_install)) = diff {
		if to_uninstall.is_empty() && to_install.is_empty() {
			return future::Either::A(future::ok(true));
		}

		future::Either::B(
			::util::ensure_user_credentials(local_api, web_api)
			.and_then(move |user_credentials| {
				for installed_mod in to_uninstall {
					match installed_mod.mod_type() {
						::factorio_mods_local::InstalledModType::Zipped => {
							let path = installed_mod.path();
							println!("Removing file {}", path.display());
							if let Err(err) = ::std::fs::remove_file(path) {
								return future::Either::A(Err(err).chain_err(|| format!("Could not remove file {}", path.display())).into_future());
							}
						},

						::factorio_mods_local::InstalledModType::Unpacked => {
							let path = installed_mod.path();
							println!("Removing directory {}", path.display());
							if let Err(err) = ::std::fs::remove_dir_all(path) {
								return future::Either::A(Err(err).chain_err(|| format!("Could not remove directory {}", path.display())).into_future());
							}
						},
					}
				}

				let mods_directory = local_api.mods_directory();
				let mods_directory_canonicalized = match mods_directory.canonicalize() {
					Ok(mods_directory_canonicalized) =>
						mods_directory_canonicalized,

					Err(err) =>
						return future::Either::A(Err(err).chain_err(|| format!("Could not canonicalize {}", mods_directory.display())).into_future()),
				};

				future::Either::B(
					future::join_all(
						to_install
						.into_iter()
						.map(move |release| {
							let filename = mods_directory.join(release.filename());
							let displayable_filename = filename.display().to_string();

							let mut download_filename: ::std::ffi::OsString = if let Some(filename) = filename.file_name() {
								filename.into()
							}
							else {
								return future::Either::A(future::err(format!("Could not parse filename {}", displayable_filename).into()));
							};

							download_filename.push(".new");
							let download_filename = filename.with_file_name(download_filename);
							let download_displayable_filename = download_filename.display().to_string();

							println!("Downloading to {}", download_displayable_filename);

							{
								let parent = if let Some(parent) = download_filename.parent() {
									parent
								}
								else {
									return future::Either::A(future::err(format!("Filename {} is malformed", download_displayable_filename).into()));
								};

								let parent_canonicalized = match parent.canonicalize() {
									Ok(parent_canonicalized) =>
										parent_canonicalized,

									Err(err) =>
										return future::Either::A(Err(err).chain_err(|| format!("Filename {} is malformed", download_displayable_filename)).into_future()),
								};

								if parent_canonicalized != mods_directory_canonicalized {
									return future::Either::A(future::err(format!("Filename {} is malformed", download_displayable_filename).into()));
								}
							}

							let mut file = ::std::fs::OpenOptions::new();
							let file = file.create(true).truncate(true).write(true);
							let file = match file.open(&download_filename) {
								Ok(file) =>
									file,
								Err(err) =>
									return future::Either::A(Err(err).chain_err(|| format!("Could not open {} for writing", download_displayable_filename)).into_future()),
							};

							future::Either::B(DownloadFileFuture {
								chunk_stream: web_api.download(&release, &user_credentials),
								writer: ::std::io::BufWriter::new(file),
								release_name: release.info_json().name().clone(),
								release_version: release.version().clone(),
								filename,
								displayable_filename,
								download_filename,
								download_displayable_filename,
							})
						}))
					.map(|_| true))
			}))
	}
	else {
		future::Either::A(future::ok(false))
	})
}

struct DownloadFileFuture<S> {
	chunk_stream: S,
	writer: ::std::io::BufWriter<::std::fs::File>,
	release_name: ::factorio_mods_common::ModName,
	release_version: ::factorio_mods_common::ReleaseVersion,
	filename: ::std::path::PathBuf,
	displayable_filename: String,
	download_filename: ::std::path::PathBuf,
	download_displayable_filename: String,
}

impl<S> Future for DownloadFileFuture<S> where S: Stream<Item = ::factorio_mods_web::reqwest::unstable::async::Chunk, Error = ::factorio_mods_web::Error> {
	type Item = ();
	type Error = ::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		loop {
			match self.chunk_stream.poll() {
				Ok(Async::Ready(Some(chunk))) => match ::std::io::Write::write_all(&mut self.writer, &chunk) {
					Ok(()) => (),
					Err(err) => return Err(err).chain_err(|| format!("Could not write to file {}", self.download_displayable_filename.clone())),
				},

				Ok(Async::Ready(None)) => {
					println!("Renaming {} to {}", self.download_displayable_filename, self.displayable_filename);
					match ::std::fs::rename(&self.download_filename, &self.filename) {
						Ok(()) => return Ok(Async::Ready(())),
						Err(err) => return Err(err).chain_err(|| format!("Could not rename {} to {}", self.download_displayable_filename, self.displayable_filename))
					}
				},

				Ok(Async::NotReady) =>
					return Ok(Async::NotReady),

				Err(err) =>
					return Err(err).chain_err(|| format!("Could not download release {} {}", self.release_name, self.release_version))
			};
		}
	}
}

#[derive(Debug)]
struct Cache<E> {
	graph: ::petgraph::Graph<Installable, E>,
	name_to_node_indices: ::multimap::MultiMap<::factorio_mods_common::ModName, ::petgraph::graph::NodeIndex>,
}

impl<E> Default for Cache<E> {
	fn default() -> Self {
		Cache {
			graph: Default::default(),
			name_to_node_indices: Default::default(),
		}
	}
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
) -> impl Future<Item = Option<::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_web::ModRelease>>, Error = ::Error> + 'a {
	let cache = ::futures_mutex::FutMutex::new(Default::default());

	add_installable(cache, Installable::Base(::factorio_mods_common::ModName::new("base".to_string()), game_version.clone()))
	.and_then(move |cache| {
		println!("Fetching releases...");

		let futures: Vec<_> =
			reqs.keys()
			.map(|name| add_mod(api, game_version, cache.clone(), name.clone()))
			.collect();
		future::join_all(futures)
		.map(|_| (reqs, cache))
	})
	.and_then(|(reqs, cache)|
		cache.lock()
		.then(move |guard| match guard {
			Ok(mut guard) => {
				let graph = ::std::mem::replace(&mut (*guard).graph, Default::default());
				compute_solution(graph, &reqs)
			},

			Err(()) =>
				unreachable!(),
		}))
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
				println!("{} {} -> {}", installed_mod.info().name(), installed_mod.info().version(), release.version());
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
			println!("{} {}", installed_mod.info().name(), installed_mod.info().version());
		}
	}

	if !to_install.is_empty() {
		println!();
		println!("The following new mods will be installed:");
		for release in &to_install {
			println!("{} {}", release.info_json().name(), release.version());
		}
	}

	println!();

	if to_uninstall.is_empty() && to_install.is_empty() {
		println!("Nothing to do.");
	}
	else {
		match ::util::prompt_continue() {
			Ok(true) => (),
			Ok(false) => return Ok(None),
			Err(err) => return Err(err),
		}
	}

	Ok(Some((to_uninstall, to_install)))
}

fn add_mod<'a, E>(
	api: &'a ::factorio_mods_web::API,
	game_version: &'a ::factorio_mods_common::ReleaseVersion,
	cache: ::futures_mutex::FutMutex<Cache<E>>,
	name: ::factorio_mods_common::ModName,
) -> Box<Future<Item = (), Error = ::Error> + 'a> where E: 'a {
	Box::new(
		cache.lock()
		.then(|guard| match guard {
			Ok(mut guard) => {
				let need_to_fetch = {
					let cache = &mut *guard;

					match cache.name_to_node_indices.entry(name.clone()) {
						::multimap::Entry::Occupied(_) => false,
						::multimap::Entry::Vacant(entry) => {
							entry.insert_vec(vec![]);
							true
						},
					}
				};

				Ok((need_to_fetch, guard.unlock(), name))
			},

			Err(()) =>
				unreachable!(),
		})
		.and_then(move |(need_to_fetch, cache, name)| {
			if !need_to_fetch {
				return future::Either::A(future::ok(()));
			}

			println!("    {} ...", name);

			future::Either::B(
				api.get(&name)
				.then(move |result| match result {
					Ok(mod_) => {
						let add_releases_and_deps_futures: Vec<_> =
							mod_.releases().into_iter()
							.flat_map(|release| if release.factorio_version().matches(game_version) {
								let mut futures: Vec<_> =
									release.info_json().dependencies()
									.into_iter()
									.filter_map(|dep| if dep.required() {
										Some(add_mod(api, game_version, cache.clone(), dep.name().clone()))
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

						future::Either::A(
							future::join_all(add_releases_and_deps_futures)
							.map(|_| ()))
					},

					Err(err) => match *err.kind() {
						::factorio_mods_web::ErrorKind::StatusCode(_, ::factorio_mods_web::reqwest::StatusCode::NotFound) => future::Either::B(future::ok(())),

						_ => future::Either::B(Err(err).chain_err(|| format!("Could not get mod info for {}", name)).into_future()),
					}
				}))
		}))
}

fn add_installable<E>(
	cache: ::futures_mutex::FutMutex<Cache<E>>,
	installable: Installable,
) -> impl Future<Item = ::futures_mutex::FutMutex<Cache<E>>, Error = ::Error> {
	cache.lock()
	.then(|guard| match guard {
		Ok(mut guard) => {
			{
				let cache = &mut *guard;
				cache.name_to_node_indices.insert(installable.name().clone(), cache.graph.add_node(installable));
			}

			Ok(guard.unlock())
		},

		Err(()) =>
			unreachable!(),
	})
}

fn compute_solution(
	mut graph: ::petgraph::Graph<Installable, Relation>,
	reqs: &::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
) -> ::Result<Option<::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_web::ModRelease>>> {
	println!("Computing solution...");

	let mut edges_to_add = vec![];

	for node_index1 in graph.node_indices() {
		let installable1 = &graph[node_index1];

		for node_index2 in graph.node_indices() {
			if node_index1 == node_index2 {
				continue
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
						continue
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
			break
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
