lazy_static! {
	static ref REQUIREMENT_REGEX: ::regex::Regex = ::regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap();
}

pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Install (or update) mods.")
			(@arg requirements: ... +required index(1) "requirements to install"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();
		let local_api = local_api();

		let requirements = matches.values_of("requirements").unwrap();

		let user_credentials = match local_api.user_credentials() {
			Ok(user_credentials) => user_credentials,

			Err(err) => (|| {
				if let ::factorio_mods_local::ErrorKind::IncompleteUserCredentials(ref existing_username) = *err.kind() {
					loop {
						println!("You need a Factorio account to download mods.");
						println!("Please provide your username and password to authenticate yourself.");
						match *existing_username {
							Some(ref username) => print!("Username [{}]: ", username),
							None => print!("Username: "),
						}
						let stdout = ::std::io::stdout();
						::std::io::Write::flush(&mut stdout.lock()).unwrap();

						let mut username = String::new();
						::std::io::stdin().read_line(&mut username).unwrap();
						let username = username.trim().to_string();
						let username = match(username.is_empty(), existing_username) {
							(false, _) => ::std::borrow::Cow::Owned(::factorio_mods_common::ServiceUsername::new(username)),
							(true, &Some(ref username)) => ::std::borrow::Cow::Borrowed(username),
							_ => continue,
						};
						let password = ::rpassword::prompt_password_stdout("Password (not shown): ").unwrap();

						match web_api.login(username.into_owned(), &password) {
							Ok(user_credentials) => {
								println!("Logged in successfully.");
								return user_credentials;
							},

							Err(err) => {
								match err.kind() {
									&::factorio_mods_web::ErrorKind::LoginFailure(ref message) => println!("Authentication error: {}", message),
									k => println!("Error: {}", k),
								}

								continue;
							},
						}
					}
				}

				panic!(err);
			})(),
		};

		let config = ::config::Config::load(&local_api);
		let mut reqs = config.mods().clone();
		reqs.extend(requirements.into_iter().map(|requirement| {
			let captures = REQUIREMENT_REGEX.captures(requirement).unwrap();
			let name = ::factorio_mods_common::ModName::new(captures[1].to_string());
			let requirement_string = captures.at(2).unwrap_or("*");
			let requirement = ::semver::VersionReq::parse(requirement_string).unwrap();
			(name.clone(), ::config::ModVersionReq::new(requirement))
		}));
		let game_version = local_api.game_version().clone();
		if let Some(solution) = ::solve::solve(&web_api, game_version, &reqs) {
			let config = config.with_mods(reqs);
			config.save();

			let all_installed_mods: &::multimap::MultiMap<_, _> =
				&local_api.installed_mods().unwrap().map(|mod_| {
					let mod_ = mod_.unwrap();
					(mod_.info().name().clone(), mod_)
				}).collect();

			let mut to_keep = vec![];
			let mut to_uninstall = vec![];
			let mut to_install = vec![];
			let mut to_upgrade = vec![];

			for (name, installed_mods) in all_installed_mods.iter_all() {
				if let Some(release) = solution.get(name) {
					for installed_mod in installed_mods {
						if installed_mod.info().version() == release.version() {
							to_keep.push(installed_mod);
						}
						else {
							to_uninstall.push(installed_mod);
							to_upgrade.push((installed_mod, release));
						}
					}
				}
				else {
					to_uninstall.extend(installed_mods);
				}
			}

			for (name, release) in &solution {
				if let Some(installed_mods) = all_installed_mods.get_vec(name) {
					if !installed_mods.iter().any(|installed_mod| installed_mod.info().version() == release.version()) {
						to_install.push(release);
					}
				}
				else {
					to_install.push(release);
				}
			}

			to_uninstall.sort_by(|installed_mod1, installed_mod2|
				installed_mod1.info().name().cmp(installed_mod2.info().name())
				.then_with(|| installed_mod1.info().version().cmp(installed_mod2.info().version())));
			to_install.sort_by(|release1, release2|
				release1.info_json().name().cmp(release2.info_json().name())
				.then_with(|| release1.version().cmp(release2.version())));
			to_upgrade.sort_by(|&(installed_mod1, release1), &(installed_mod2, release2)|
				installed_mod1.info().name().cmp(installed_mod2.info().name())
				.then_with(|| installed_mod1.info().version().cmp(installed_mod2.info().version()))
				.then_with(|| release1.info_json().name().cmp(release2.info_json().name()))
				.then_with(|| release1.version().cmp(release2.version())));

			if !to_upgrade.is_empty() {
				println!();
				println!("The following mods will be upgraded:");
				for &(installed_mod, release) in &to_upgrade {
					println!("{} {} -> {}", installed_mod.info().name(), installed_mod.info().version(), release.version());
				}
			}

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
				return
			}
			else {
				let mut choice = String::new();

				loop {
					print!("Continue? [y/n]: ");

					let stdout = ::std::io::stdout();
					::std::io::Write::flush(&mut stdout.lock()).unwrap();

					::std::io::stdin().read_line(&mut choice).unwrap();

					match choice.trim() {
						"y" | "Y" => break,
						"n" | "N" => return,
						_ => continue,
					}
				}
			}

			for installed_mod in to_uninstall {
				match *installed_mod.mod_type() {
					::factorio_mods_local::InstalledModType::Zipped => {
						let path = installed_mod.path();
						println!("Removing file {}", path.display());
						::std::fs::remove_file(path).unwrap();
					},

					::factorio_mods_local::InstalledModType::Unpacked => {
						let path = installed_mod.path();
						println!("Removing directory {}", path.display());
						::std::fs::remove_dir_all(path).unwrap();
					},
				}
			}

			let mods_directory = local_api.mods_directory();

			for release in to_install {
				let filename = mods_directory.join(release.filename());
				let displayable_filename = filename.display().to_string();

				let mut download_filename: ::std::ffi::OsString = filename.file_name().unwrap().into();
				download_filename.push(".new");
				let download_filename = filename.with_file_name(download_filename);
				let download_displayable_filename = download_filename.display().to_string();

				println!("Downloading to {}", download_displayable_filename);

				if let Some(parent) = download_filename.parent() {
					if let Ok(parent_canonicalized) = parent.canonicalize() {
						if parent_canonicalized != mods_directory.canonicalize().unwrap() {
							panic!("Filename is malformed.");
						}
					}
					else {
						panic!("Filename is malformed.");
					}
				}
				else {
					panic!("Filename is malformed.");
				}

				{
					let mut reader = ::std::io::BufReader::new(web_api.download(release, &user_credentials).unwrap());

					let mut file = ::std::fs::OpenOptions::new();
					let mut file = file.create(true).truncate(true);
					let file = file.write(true).open(&download_filename).unwrap();

					let mut writer = ::std::io::BufWriter::new(file);
					::std::io::copy(&mut reader, &mut writer).unwrap();
				}

				println!("Renaming {} to {}", download_displayable_filename, displayable_filename);
				::std::fs::rename(download_filename, filename).unwrap();
			}
		}
		else {
			panic!("No solution found.");
		}
	}
}
