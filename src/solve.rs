use failure::{ Fail, ResultExt };

/// Computes which old mods to uninstall and which new mods to install based on the given reqs.
/// Asks the user for confirmation, then applies the diff.
///
/// Returns true if the diff was successfully applied or empty.
pub async fn compute_and_apply_diff<'a>(
	local_api: &'a factorio_mods_local::API,
	web_api: &'a factorio_mods_web::API,
	config: crate::config::Config,
	prompt_override: Option<bool>,
) -> Result<(), failure::Error> {
	let mut config = config; // TODO: Workaround for https://github.com/rust-lang/rust/issues/60498

	let user_credentials = await!(crate::util::ensure_user_credentials(local_api, web_api, prompt_override))?;

	let game_version = local_api.game_version();

	let cache_directory = config.cache_directory()?;
	std::fs::create_dir_all(&cache_directory)
	.with_context(|_| format!("Could not create cache directory {}", cache_directory.display()))?;

	let cache_directory_canonicalized =
		cache_directory.canonicalize()
		.with_context(|_| format!("Could not canonicalize {}", cache_directory.display()))?;

	println!("Updating cache ...");

	let solution_future = SolutionFuture::new(web_api, user_credentials, game_version, config.mods, cache_directory, cache_directory_canonicalized);
	let (solution, mut reqs) = await!(solution_future)?;

	reqs.remove(&factorio_mods_common::ModName("base".to_string()));
	config.mods = reqs;

	let solution =
		solution
		.ok_or_else(|| failure::err_msg("No solution found."))?
		.into_iter()
		.filter_map(|(name, installable)|
			if let Installable::Mod(cached_mod) = installable {
				Some((name, cached_mod))
			}
			else {
				None
			})
		.collect();

	let (to_uninstall, to_install) = match compute_diff(solution, local_api, prompt_override)? {
		Some(diff) => diff,
		None => return Ok(()),
	};

	for installed_mod in to_uninstall {
		let path = installed_mod.path;

		match installed_mod.mod_type {
			factorio_mods_local::InstalledModType::Zipped => {
				println!(
					"    Removing {} {} ... removing file {} ...",
					installed_mod.info.name, installed_mod.info.version,
					path.display());
				std::fs::remove_file(&path)
				.with_context(|_| format!("Could not remove file {}", path.display()))?;
			},

			factorio_mods_local::InstalledModType::Unpacked => {
				println!(
					"    Removing {} {} ... removing directory {} ...",
					installed_mod.info.name, installed_mod.info.version,
					path.display());
				std::fs::remove_dir_all(&path)
				.with_context(|_| format!("Could not remove directory {}", path.display()))?;
			},
		}

		println!(
			"    Removing {} {} ... done",
			installed_mod.info.name, installed_mod.info.version);
	}

	let mods_directory = local_api.mods_directory();

	for cached_mod in to_install {
		let target = mods_directory.join(cached_mod.path.file_name().unwrap());

		println!("    Installing {} {} ... copying to {}", cached_mod.info.name, cached_mod.info.version, target.display());

		let _ =
			std::fs::copy(&cached_mod.path, &target)
			.with_context(|_| format!("Could not copy file {} to {}", cached_mod.path.display(), target.display()))?;

		println!("    Installing {} {} ... done", cached_mod.info.name, cached_mod.info.version);
	}

	config.save()?;

	Ok(())
}

fn compute_diff(
	mut solution: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_local::InstalledMod>,
	local_api: &factorio_mods_local::API,
	prompt_override: Option<bool>,
) -> Result<Option<(Vec<factorio_mods_local::InstalledMod>, Vec<factorio_mods_local::InstalledMod>)>, failure::Error> {
	let all_installed_mods: Result<multimap::MultiMap<_, _>, failure::Error> =
		local_api.installed_mods().context("Could not enumerate installed mods")?
		.map(|mod_| Ok(
			mod_
			.map(|mod_| (mod_.info.name.clone(), mod_))
			.context("Could not process an installed mod")?))
		.collect();

	let all_installed_mods = all_installed_mods.context("Could not enumerate installed mods")?;

	let mut to_uninstall = vec![];
	let mut to_install = std::collections::HashMap::new();

	for (name, installed_mods) in all_installed_mods {
		match solution.remove(&name) {
			Some(cached_mod) => {
				let mut already_installed = false;

				for installed_mod in installed_mods {
					if cached_mod.info.version == installed_mod.info.version {
						already_installed = true;
					}
					else {
						to_uninstall.push(installed_mod);
					}
				}

				if !already_installed {
					to_install.insert(name, cached_mod);
				}
			},

			None =>
				to_uninstall.extend(installed_mods),
		}
	}

	to_install.extend(solution);

	{
		let to_upgrade: Vec<_> =
			itertools::Itertools::sorted_by(
				to_uninstall.iter().filter_map(|installed_mod|
					to_install.get(&installed_mod.info.name)
					.map(|cached_mod| (installed_mod, cached_mod))),
				|&(installed_mod1, cached_mod1), &(installed_mod2, cached_mod2)|
					installed_mod1.info.name.cmp(&installed_mod2.info.name)
					.then_with(|| installed_mod1.info.version.cmp(&installed_mod2.info.version))
					.then_with(|| cached_mod1.info.name.cmp(&cached_mod2.info.name))
					.then_with(|| cached_mod1.info.version.cmp(&cached_mod2.info.version)))
			.collect();

		if !to_upgrade.is_empty() {
			println!();
			println!("The following mods will be upgraded:");
			for (installed_mod, cached_mod) in to_upgrade {
				println!("    {} {} -> {}", installed_mod.info.name, installed_mod.info.version, cached_mod.info.version);
			}
		}
	}

	to_uninstall.sort_by(|installed_mod1, installed_mod2|
		installed_mod1.info.name.cmp(&installed_mod2.info.name)
		.then_with(|| installed_mod1.info.version.cmp(&installed_mod2.info.version)));

	let to_install: Vec<_> =
		itertools::Itertools::sorted_by(to_install.into_iter().map(|(_, cached_mod)| cached_mod), |cached_mod1, cached_mod2|
			cached_mod1.info.name.cmp(&cached_mod2.info.name)
			.then_with(|| cached_mod1.info.version.cmp(&cached_mod2.info.version)))
		.collect();

	if !to_uninstall.is_empty() {
		println!();
		println!("The following mods will be removed:");
		for installed_mod in &to_uninstall {
			println!("    {} {}", installed_mod.info.name, installed_mod.info.version);
		}
	}

	if !to_install.is_empty() {
		println!();
		println!("The following new mods will be installed:");
		for cached_mod in &to_install {
			println!("    {} {}", cached_mod.info.name, cached_mod.info.version);
		}
	}

	println!();

	if to_uninstall.is_empty() && to_install.is_empty() {
		println!("Nothing to do.");
	}
	else if !crate::util::prompt_continue(prompt_override)? {
		return Ok(None);
	}

	Ok(Some((to_uninstall, to_install)))
}

struct SolutionFuture<'a> {
	packages: Vec<Installable>,
	already_fetching: std::collections::HashSet<std::rc::Rc<factorio_mods_common::ModName>>,
	pending: Vec<CacheFuture>,
	web_api: &'a factorio_mods_web::API,
	user_credentials: factorio_mods_common::UserCredentials,
	game_version: &'a factorio_mods_common::ReleaseVersion,
	reqs: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
	cache_directory: std::path::PathBuf,
	cache_directory_canonicalized: std::path::PathBuf,
}

impl<'a> SolutionFuture<'a> {
	fn new(
		web_api: &'a factorio_mods_web::API,
		user_credentials: factorio_mods_common::UserCredentials,
		game_version: &'a factorio_mods_common::ReleaseVersion,
		mut reqs: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
		cache_directory: std::path::PathBuf,
		cache_directory_canonicalized: std::path::PathBuf,
	) -> Self {
		let packages = vec![Installable::Base(factorio_mods_common::ModName("base".to_string()), game_version.clone())];

		let mut result = SolutionFuture {
			packages,
			already_fetching: Default::default(),
			pending: Default::default(),
			web_api,
			user_credentials,
			game_version,
			reqs: Default::default(),
			cache_directory,
			cache_directory_canonicalized,
		};

		for mod_name in reqs.keys() {
			get(mod_name.clone().into(), &mut result.already_fetching, &mut result.pending, web_api);
		}

		reqs.insert(factorio_mods_common::ModName("base".to_string()), factorio_mods_common::ModVersionReq(semver::VersionReq::exact(&game_version.0)));

		result.reqs = reqs;

		result
	}
}

impl std::future::Future for SolutionFuture<'_> {
	type Output = Result<(
		Option<std::collections::HashMap<factorio_mods_common::ModName, Installable>>,
		std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
	), failure::Error>;

	fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		let this = &mut *self;

		let mut i = 0;

		while i < this.pending.len() {
			let mut new = vec![];

			match &mut this.pending[i] {
				CacheFuture::Get(get) => match get {
					Some((mod_name, f)) => match f.as_mut().poll(cx) {
						std::task::Poll::Pending => (),

						std::task::Poll::Ready(Ok(mod_)) => {
							let (mod_name, _) = get.take().unwrap();

							println!("    Getting {} ... done", mod_name);

							for release in mod_.releases {
								if !release.info_json.factorio_version.0.matches(&this.game_version.0) {
									continue;
								}

								println!("        Downloading {} {} ... downloading to cache", mod_name, release.version);

								let filename = this.cache_directory.join(&release.filename.0);
								let displayable_filename = filename.display().to_string();

								if filename.exists() {
									println!("        Downloading {} {} ... parsing", mod_name, release.version);
									parse_cached_mod(filename, &displayable_filename, &mut this.already_fetching, &mut new, &mut this.packages, this.web_api)?;
									continue;
								}

								let mut download_filename: std::ffi::OsString =
									filename.file_name()
									.ok_or_else(|| failure::err_msg(format!("Could not parse filename {}", displayable_filename)))?
									.into();

								download_filename.push(".new");
								let download_filename = filename.with_file_name(download_filename);
								let download_displayable_filename = download_filename.display().to_string();

								{
									let parent =
										download_filename.parent().ok_or_else(||
											failure::err_msg(format!("Filename {} is malformed", download_displayable_filename)))?;
									let parent_canonicalized =
										parent.canonicalize()
										.with_context(|_| format!("Filename {} is malformed", download_displayable_filename))?;
									if parent_canonicalized != this.cache_directory_canonicalized {
										return std::task::Poll::Ready(Err(failure::err_msg(format!("Filename {} is malformed", download_displayable_filename))));
									}
								}

								let mut download_file = std::fs::OpenOptions::new();
								let download_file = download_file.create(true).truncate(true).write(true);
								let download_file =
									download_file.open(&download_filename)
									.with_context(|_| format!("Could not open {} for writing", download_displayable_filename))?;
								let download_file = std::io::BufWriter::new(download_file);

								let chunk_stream = this.web_api.download(&release, &this.user_credentials);

								new.push(CacheFuture::Download(Some(DownloadFuture {
									mod_name: mod_name.clone(),
									release_version: release.version,
									chunk_stream: Box::pin(chunk_stream),
									download_file,
									download_filename,
									download_displayable_filename,
									filename,
									displayable_filename,
								})));
							}
						},

						std::task::Poll::Ready(Err(err)) => match *err.kind() {
							// Don't fail the whole process due to non-existent deps. Releases with unmet deps will be handled when computing the solution.
							factorio_mods_web::ErrorKind::StatusCode(_, crate::reqwest::StatusCode::NOT_FOUND) => {
								let _ = get.take();
							},

							_ => Err(err).with_context(|_| format!("Could not get mod info for {}", mod_name))?,
						},
					},

					None => unreachable!(),
				},

				CacheFuture::Download(download) => match download {
					Some(f) => loop {
						match <_ as futures_core::Stream>::poll_next(f.chunk_stream.as_mut(), cx) {
							std::task::Poll::Pending => break,

							std::task::Poll::Ready(Some(Ok(chunk))) =>
								std::io::Write::write_all(&mut f.download_file, &chunk)
								.with_context(|_| format!("Could not write to file {}", f.download_displayable_filename))?,

							std::task::Poll::Ready(Some(Err(err))) =>
								return std::task::Poll::Ready(
									Err(err.context(format!("Could not download release {} {}", f.mod_name, f.release_version))
									.into())),

							std::task::Poll::Ready(None) => {
								let DownloadFuture {
									mod_name,
									release_version,
									mut download_file,
									download_filename,
									download_displayable_filename,
									filename,
									displayable_filename,
									..
								} = download.take().unwrap();

								println!("        Downloading {} {} ... parsing", mod_name, release_version);

								std::io::Write::flush(&mut download_file)
								.with_context(|_| format!("Could not write to file {}", download_displayable_filename))?;
								drop(download_file);

								std::fs::rename(&download_filename, &filename)
								.with_context(|_| format!("Could not rename {} to {}", download_displayable_filename, displayable_filename))?;

								parse_cached_mod(filename.clone(), &displayable_filename, &mut this.already_fetching, &mut new, &mut this.packages, this.web_api)?;

								println!("        Downloading {} {} ... done", mod_name, release_version);

								break;
							},
						}
					},

					None => unreachable!(),
				},
			}

			i += 1;

			this.pending.extend(new);
		}

		this.pending.retain(|f| match f {
			CacheFuture::Get(None) | CacheFuture::Download(None) => false,
			_ => true,
		});

		if !this.pending.is_empty() {
			return std::task::Poll::Pending;
		}

		println!("Updating cache ... done");

		let packages = std::mem::replace(&mut this.packages, Default::default());
		let reqs = std::mem::replace(&mut this.reqs, Default::default());

		println!();
		println!("Computing solution...");

		let solution =
			package::compute_solution(packages, &reqs)
			.context("Could not compute solution.")?;

		std::task::Poll::Ready(Ok((solution, reqs)))
	}
}

fn get(
	mod_name: std::rc::Rc<factorio_mods_common::ModName>,
	already_fetching: &mut std::collections::HashSet<std::rc::Rc<factorio_mods_common::ModName>>,
	new: &mut Vec<CacheFuture>,
	web_api: &factorio_mods_web::API,
) {
	if already_fetching.insert(mod_name.clone()) {
		println!("    Getting {} ...", mod_name);

		let f = Box::pin(web_api.get(&mod_name));
		new.push(CacheFuture::Get(Some((mod_name, f))));
	}
}

fn parse_cached_mod(
	filename: std::path::PathBuf,
	displayable_filename: &str,
	already_fetching: &mut std::collections::HashSet<std::rc::Rc<factorio_mods_common::ModName>>,
	new: &mut Vec<CacheFuture>,
	packages: &mut Vec<Installable>,
	web_api: &factorio_mods_web::API,
) -> Result<(), failure::Error> {
	let cached_mod =
		factorio_mods_local::InstalledMod::parse(filename)
		.with_context(|_| format!("Could not parse {}", displayable_filename))?;

	for dep in cached_mod.info.dependencies.iter().filter(|dep| dep.required && dep.name.0 != "base") {
		get(dep.name.clone().into(), already_fetching, new, web_api);
	}

	packages.push(Installable::Mod(cached_mod));

	Ok(())
}

enum CacheFuture {
	Get(Option<(std::rc::Rc<factorio_mods_common::ModName>, std::pin::Pin<Box<factorio_mods_web::GetResponse>>)>),
	Download(Option<DownloadFuture>),
}

struct DownloadFuture {
	mod_name: std::rc::Rc<factorio_mods_common::ModName>,
	release_version: factorio_mods_common::ReleaseVersion,
	chunk_stream: std::pin::Pin<Box<factorio_mods_web::DownloadResponse>>,
	download_file: std::io::BufWriter<std::fs::File>,
	download_filename: std::path::PathBuf,
	download_displayable_filename: String,
	filename: std::path::PathBuf,
	displayable_filename: String,
}

#[derive(Clone, Debug)]
enum Installable {
	Base(factorio_mods_common::ModName, factorio_mods_common::ReleaseVersion),
	Mod(factorio_mods_local::InstalledMod),
}

impl package::Package for Installable {
	type Name = factorio_mods_common::ModName;
	type Version = factorio_mods_common::ReleaseVersion;
	type Dependency = factorio_mods_common::Dependency;

	fn name(&self) -> &Self::Name {
		match *self {
			Installable::Base(ref name, _) => name,
			Installable::Mod(ref cached_mod) => &cached_mod.info.name,
		}
	}

	fn version(&self) -> &Self::Version {
		match *self {
			Installable::Base(_, ref version) => version,
			Installable::Mod(ref cached_mod) => &cached_mod.info.version,
		}
	}

	fn dependencies(&self) -> &[Self::Dependency] {
		match *self {
			Installable::Base(..) => &[],
			Installable::Mod(ref cached_mod) => &cached_mod.info.dependencies,
		}
	}
}
