#[derive(Debug)]
struct ModVersionReq(::semver::VersionReq);

lazy_static! {
	static ref REQUIREMENT_REGEX: ::regex::Regex = ::regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap();
}

pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("Install (or update) mods.")
			.arg(
				::clap::Arg::with_name("reinstall")
					.help("allow reinstalling mods")
					.long("reinstall")
					.short("R"))
			.arg(
				::clap::Arg::with_name("requirements")
					.help("requirements to install")
					.index(1)
					.multiple(true)
					.required(true))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();
		let local_api = local_api();

		let reinstall = matches.is_present("reinstall");
		let requirements = matches.values_of("requirements").unwrap();

		let user_credentials = match local_api.user_credentials() {
			Ok(user_credentials) => user_credentials,

			Err(err) => || -> ::factorio_mods_common::UserCredentials {
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
			}(),
		};

		for requirement in requirements {
			let captures = REQUIREMENT_REGEX.captures(requirement).unwrap();
			let name = ::factorio_mods_common::ModName::new(captures[1].to_string());
			let requirement_string = captures.at(2).unwrap_or("*");
			let requirement = ModVersionReq(::semver::VersionReq::parse(requirement_string).unwrap());

			let mod_ = web_api.get(name.clone()).unwrap();

			let mut releases = mod_.releases().to_vec();
			releases.sort_by(|r1, r2| r2.version().cmp(r1.version()));
			let releases = releases;
			let best_release = releases.iter().find(|release| requirement.0.matches(release.version()));
			if let Some(best_release) = best_release {
				let mods_directory = local_api.mods_directory();
				let expected_file_name = mods_directory.join(&**best_release.file_name());
				let expected_file_name = expected_file_name.display();
				println!("Saving to: {}", expected_file_name);
				if let Err(err) = web_api.download(best_release, mods_directory, &user_credentials, reinstall) {
					match *err.kind() {
						::factorio_mods_web::ErrorKind::IO(ref err) if err.kind() == ::std::io::ErrorKind::AlreadyExists => {
							println!("File {} already exists. Use -R to replace it.", expected_file_name);
							continue;
						},

						_ => { },
					}

					panic!(err);
				}
			}
			else {
				println!("No match found for {}{}", name, requirement_string);
			}
		}
	}
}
