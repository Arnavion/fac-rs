mod web_reader;
mod zip;

use failure::{ Fail, ResultExt };

/// Computes which old mods to uninstall and which new mods to install based on the given reqs.
/// Asks the user for confirmation, then applies the diff.
///
/// Returns true if the diff was successfully applied or empty.
pub async fn compute_and_apply_diff<'a>(
	local_api: &'a factorio_mods_local::API,
	web_api: &'a factorio_mods_web::API,
	mut config: crate::config::Config,
	prompt_override: Option<bool>,
) -> Result<(), failure::Error> {
	let user_credentials = std::rc::Rc::new(crate::util::ensure_user_credentials(local_api, web_api, prompt_override).await?);

	let game_version = local_api.game_version();

	println!("Getting mod information ...");

	let solution_future = SolutionFuture::new(web_api, user_credentials.clone(), game_version, config.mods);
	let (solution, mut reqs) = solution_future.await?;

	let _ = reqs.remove(&factorio_mods_common::ModName("base".to_string()));
	config.mods = reqs;

	let solution =
		solution
		.ok_or_else(|| failure::err_msg("No solution found."))?
		.into_iter()
		.filter_map(|(name, installable)|
			if let Installable::Mod(_, release, _) = installable {
				Some((name, release))
			}
			else {
				None
			})
		.collect();

	let (to_uninstall, to_install) = match compute_diff(solution, local_api, prompt_override)? {
		Some(diff) => diff,
		None => return Ok(()),
	};

	let mods_directory = local_api.mods_directory();
	std::fs::create_dir_all(&mods_directory)
		.with_context(|_| format!("Could not create mods directory {}", mods_directory.display()))?;

	let mods_directory_canonicalized =
		mods_directory.canonicalize()
		.with_context(|_| format!("Could not canonicalize {}", mods_directory.display()))?;

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

	let download_futures: futures_util::stream::FuturesUnordered<_> =
		to_install.into_iter()
		.map(move |(name, release)|
			download_mod(
				web_api,
				name,
				release,
				mods_directory,
				&mods_directory_canonicalized,
				&user_credentials))
		.collect();
	futures_util::try_stream::TryStreamExt::try_for_each_concurrent(download_futures, None, futures_util::future::ok).await?;

	config.save()?;

	Ok(())
}

fn download_mod(
	web_api: &factorio_mods_web::API,
	mod_name: factorio_mods_common::ModName,
	release: std::rc::Rc<factorio_mods_web::ModRelease>,
	mods_directory: &std::path::Path,
	mods_directory_canonicalized: &std::path::Path,
	user_credentials: &factorio_mods_common::UserCredentials,
) -> impl std::future::Future<Output = Result<(), failure::Error>> + 'static {
	let target = mods_directory.join(&release.filename.0);
	let displayable_target = target.display().to_string();

	let mut download_filename: std::ffi::OsString = match target.file_name() {
		Some(download_filename) => download_filename.to_owned(),
		None =>
			return futures_util::future::Either::Left(futures_util::future::err(
				failure::err_msg(format!("Filename {} is malformed", displayable_target)))),
	};
	download_filename.push(".new");
	let download_target = target.with_file_name(download_filename);
	let download_displayable_target = download_target.display().to_string();

	{
		let is_valid =
			download_target.parent()
				.and_then(|parent|
					parent.canonicalize()
					.ok())
				.map(|parent_canonicalized| parent_canonicalized == mods_directory_canonicalized);
		match is_valid {
			Some(true) => (),
			_ =>
				return futures_util::future::Either::Left(futures_util::future::err(
					failure::err_msg(format!("Filename {} is malformed", download_displayable_target)))),
		}
	}

	println!("    Installing {} {} ... downloading to {} ...", mod_name, release.version, download_displayable_target);

	let mut chunk_stream = web_api.download(&release, &user_credentials, None);

	let download_file =
		std::fs::OpenOptions::new()
		.create(true).truncate(true).write(true)
		.open(&download_target);

	futures_util::future::Either::Right(async move {
		let download_file = download_file.with_context(|_| format!("Could not open {} for writing", download_displayable_target))?;
		let mut download_file = std::io::BufWriter::new(download_file);

		while let Some(chunk) = futures_util::stream::StreamExt::next(&mut chunk_stream).await {
			let chunk = chunk?;

			std::io::Write::write_all(&mut download_file, &chunk)
				.with_context(|_| format!("Could not write to file {}", download_displayable_target))?;
		}

		std::io::Write::flush(&mut download_file)
			.with_context(|_| format!("Could not write to file {}", download_displayable_target))?;

		drop(download_file);

		println!("    Installing {} {} ... renaming {} to {}", mod_name, release.version, download_displayable_target, displayable_target);

		std::fs::rename(&download_target, &target)
			.with_context(|_| format!("Could not rename {} to {}", download_displayable_target, displayable_target))?;

		println!("    Installing {} {} ... done", mod_name, release.version);

		Ok(())
	})
}

fn compute_diff(
	mut solution: std::collections::HashMap<factorio_mods_common::ModName, std::rc::Rc<factorio_mods_web::ModRelease>>,
	local_api: &factorio_mods_local::API,
	prompt_override: Option<bool>,
) -> Result<Option<(Vec<factorio_mods_local::InstalledMod>, Vec<(factorio_mods_common::ModName, std::rc::Rc<factorio_mods_web::ModRelease>)>)>, failure::Error> {
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
			Some(release) => {
				let mut already_installed = false;

				for installed_mod in installed_mods {
					if release.version == installed_mod.info.version {
						already_installed = true;
					}
					else {
						to_uninstall.push(installed_mod);
					}
				}

				if !already_installed {
					to_install.insert(name, release);
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
					.map(|release| (installed_mod, release))),
				|&(installed_mod1, release1), &(installed_mod2, release2)|
					installed_mod1.info.name.cmp(&installed_mod2.info.name)
					.then_with(|| installed_mod1.info.version.cmp(&installed_mod2.info.version))
					.then_with(|| release1.version.cmp(&release2.version)))
			.collect();

		if !to_upgrade.is_empty() {
			println!();
			println!("The following mods will be upgraded:");
			for (installed_mod, release) in to_upgrade {
				println!("    {} {} -> {}", installed_mod.info.name, installed_mod.info.version, release.version);
			}
		}
	}

	to_uninstall.sort_by(|installed_mod1, installed_mod2|
		installed_mod1.info.name.cmp(&installed_mod2.info.name)
		.then_with(|| installed_mod1.info.version.cmp(&installed_mod2.info.version)));

	let to_install: Vec<_> =
		itertools::Itertools::sorted_by(to_install.into_iter(), |(name1, release1), (name2, release2)|
			name1.cmp(&name2)
			.then_with(|| release1.version.cmp(&release2.version)))
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
		for (name, release) in &to_install {
			println!("    {} {}", name, release.version);
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
	pending: Vec<CacheFuture<'a>>,
	web_api: &'a factorio_mods_web::API,
	user_credentials: std::rc::Rc<factorio_mods_common::UserCredentials>,
	game_version: &'a factorio_mods_common::ReleaseVersion,
	reqs: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
}

impl<'a> SolutionFuture<'a> {
	fn new(
		web_api: &'a factorio_mods_web::API,
		user_credentials: std::rc::Rc<factorio_mods_common::UserCredentials>,
		game_version: &'a factorio_mods_common::ReleaseVersion,
		mut reqs: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
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
		};

		for mod_name in reqs.keys() {
			get(mod_name.clone().into(), &mut result.already_fetching, &mut result.pending, web_api);
		}

		reqs.insert(factorio_mods_common::ModName("base".to_string()), factorio_mods_common::ModVersionReq(semver::VersionReq::exact(&game_version.0)));

		result.reqs = reqs;

		result
	}
}

impl<'a> std::future::Future for SolutionFuture<'a> {
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
				CacheFuture::GetMod(get_mod) => match get_mod {
					Some((mod_name, f)) => match f.as_mut().poll(cx) {
						std::task::Poll::Pending => (),

						std::task::Poll::Ready(Ok(mod_)) => {
							let (mod_name, _) = get_mod.take().unwrap();

							println!("    Getting {} ... done", mod_name);

							for release in mod_.releases {
								if !release.info_json.factorio_version.0.matches(&this.game_version.0) {
									continue;
								}

								let release = std::rc::Rc::new(release);

								println!("        Getting {} {} info.json ...", mod_name, release.version);

								let web_api = this.web_api;
								let user_credentials = this.user_credentials.clone();
								new.push(CacheFuture::GetInfoJson(Some((
									mod_name.clone(),
									release.clone(),
									Box::pin(async move {
										let mut web_reader =
											web_reader::WebReader::new(web_api, release, user_credentials).await
											.context("Could not create web reader")?;
										let mod_info =
											zip::find_info_json(&mut web_reader).await
											.context("Could not get info.json")?;
										Ok(mod_info)
									}),
								))));
							}
						},

						std::task::Poll::Ready(Err(err)) => match *err.kind() {
							// Don't fail the whole process due to non-existent deps. Releases with unmet deps will be handled when computing the solution.
							factorio_mods_web::ErrorKind::StatusCode(_, crate::reqwest::StatusCode::NOT_FOUND) => {
								let _ = get_mod.take();
							},

							_ => return std::task::Poll::Ready(Err(err.context(format!("Could not get mod info for {}", mod_name)).into())),
						},
					},

					None => unreachable!(),
				},

				CacheFuture::GetInfoJson(get_info_json) => match get_info_json {
					Some((mod_name, release, f)) => match f.as_mut().poll(cx) {
						std::task::Poll::Ready(Ok(mod_info)) => {
							let (mod_name, release, _) = get_info_json.take().unwrap();

							for dep in mod_info.dependencies.iter().filter(|dep| dep.required && dep.name.0 != "base") {
								get(dep.name.clone().into(), &mut this.already_fetching, &mut new, this.web_api);
							}

							this.packages.push(Installable::Mod(mod_info.name, release.clone(), mod_info.dependencies));

							println!("        Getting {} {} info.json ... done", mod_name, release.version);
						},

						std::task::Poll::Ready(Err(err)) =>
							return std::task::Poll::Ready(
								Err(err.context(format!("Could not download release {} {}", mod_name, release.version))
								.into())),

						std::task::Poll::Pending => (),
					},

					None => unreachable!(),
				},
			}

			i += 1;

			this.pending.extend(new);
		}

		this.pending.retain(|f| match f {
			CacheFuture::GetMod(None) | CacheFuture::GetInfoJson(None) => false,
			_ => true,
		});

		if !this.pending.is_empty() {
			return std::task::Poll::Pending;
		}

		println!("Getting mod information ... done");

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
	new: &mut Vec<CacheFuture<'_>>,
	web_api: &factorio_mods_web::API,
) {
	if already_fetching.insert(mod_name.clone()) {
		println!("    Getting {} ...", mod_name);

		let f = Box::pin(web_api.get(&mod_name));
		new.push(CacheFuture::GetMod(Some((mod_name, f))));
	}
}

enum CacheFuture<'a> {
	GetMod(Option<(std::rc::Rc<factorio_mods_common::ModName>, std::pin::Pin<Box<factorio_mods_web::GetResponse>>)>),
	GetInfoJson(Option<(
		std::rc::Rc<factorio_mods_common::ModName>,
		std::rc::Rc<factorio_mods_web::ModRelease>,
		std::pin::Pin<Box<dyn std::future::Future<Output = Result<factorio_mods_local::ModInfo, failure::Error>> + 'a>>,
	)>),
}

#[derive(Clone, Debug)]
enum Installable {
	Base(factorio_mods_common::ModName, factorio_mods_common::ReleaseVersion),
	Mod(factorio_mods_common::ModName, std::rc::Rc<factorio_mods_web::ModRelease>, Vec<factorio_mods_common::Dependency>),
}

impl package::Package for Installable {
	type Name = factorio_mods_common::ModName;
	type Version = factorio_mods_common::ReleaseVersion;
	type Dependency = factorio_mods_common::Dependency;

	fn name(&self) -> &Self::Name {
		match self {
			Installable::Base(name, _) |
			Installable::Mod(name, _, _) => name,
		}
	}

	fn version(&self) -> &Self::Version {
		match self {
			Installable::Base(_, version) => version,
			Installable::Mod(_, release, _) => &release.version,
		}
	}

	fn dependencies(&self) -> &[Self::Dependency] {
		match self {
			Installable::Base(_, _) => &[],
			Installable::Mod(_, _, dependencies) => dependencies,
		}
	}
}

fn io_error_from_fail(f: &dyn Fail) -> std::io::Error {
	use std::fmt::Write;

	let mut s = String::new();

	writeln!(s, "{}", f).unwrap();

	for fail in f.iter_causes() {
		writeln!(s).unwrap();
		writeln!(s, "Caused by: {}", fail).unwrap();
	}

	if let Some(backtrace) = f.backtrace() {
		writeln!(s).unwrap();
		writeln!(s, "{}", backtrace).unwrap();
	}

	std::io::Error::new(std::io::ErrorKind::Other, s)
}