#[derive(Debug)]
struct ModVersionReq(::semver::VersionReq);

lazy_static! {
	static ref REQUIREMENT_REGEX: ::regex::Regex = ::regex::Regex::new(r"^([^@]+)(?:@(.*))?").unwrap();
}

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("Install (or update) mods.")
			.arg(
				::clap::Arg::with_name("requirements")
					.help("requirements to install")
					.index(1)
					.multiple(true)
					.required(true))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, api: ::factorio_mods_api::API, manager: ::factorio_mods_local::Manager) {
		let requirements = matches.values_of("requirements").unwrap();

		let user_credentials = match manager.user_credentials() {
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

						match api.login(username.into_owned(), &password) {
							Ok(user_credentials) => {
								println!("Logged in successfully.");
								return user_credentials;
							},

							Err(err) => {
								match err.kind() {
									&::factorio_mods_api::ErrorKind::LoginFailure(ref message) => println!("Authentication error: {}", message),
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

			let mod_ = api.get(name.clone()).unwrap();

			let mut releases = mod_.releases().to_vec();
			releases.sort_by(|r1, r2| r2.version().cmp(r1.version()));
			let releases = releases;
			let best_release = releases.iter().find(|release| requirement.0.matches(release.version()));
			if let Some(best_release) = best_release {
				let mods_directory = manager.mods_directory();
				println!("Saving to: {}", mods_directory.join(&**best_release.file_name()).display());
				api.download(best_release, mods_directory, &user_credentials).unwrap();
			}
			else {
				println!("No match found for {}{}", name, requirement_string);
			}
		}
	}
}
