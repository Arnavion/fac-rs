pub trait SubCommand<FL, FW> {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a>;
	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API;
}

pub fn wrapping_println(s: &str, indent: &str, max_width: usize) {
	let max_len = max_width - indent.len();

	let graphemes: Vec<&str> = ::unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect();
	let mut graphemes = &graphemes[..];

	loop {
		if graphemes.is_empty() {
			return;
		}

		print!("{}", indent);

		if graphemes.len() <= max_len {
			for s in graphemes {
				print!("{}", s);
			}
			println!("");
			return;
		}

		let (line, remaining) = if let Some(last_space_pos) = graphemes[..max_len].iter().rposition(|&s| s == " ") {
			(&graphemes[..last_space_pos], &graphemes[last_space_pos + 1..])
		}
		else {
			(&graphemes[..max_len], &graphemes[max_len..])
		};

		for s in line {
			print!("{}", s);
		}
		println!("");

		graphemes = remaining;
	}
}

pub fn ensure_user_credentials(local_api: &::factorio_mods_local::API, web_api: &::factorio_mods_web::API) -> ::factorio_mods_common::UserCredentials {
	match local_api.user_credentials() {
		Ok(user_credentials) => user_credentials,

		Err(err) => {
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
		},
	}
}
