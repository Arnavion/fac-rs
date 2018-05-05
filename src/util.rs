use ::futures::Future;

pub trait SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a>;
	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a>;
}

pub fn wrapping_println(s: &str, indent: &str) {
	match ::term_size::dimensions() {
		Some((width, _)) => {
			let wrapper = ::textwrap::Wrapper {
				width,
				initial_indent: indent,
				subsequent_indent: indent,
				break_words: true,
				splitter: ::textwrap::NoHyphenation,
			};

			for line in s.split('\n') {
				for line in wrapper.wrap_iter(line) {
					println!("{}", line);
				}
			}
		},

		None =>
			println!("{}{}", indent, s),
	}
}

pub fn ensure_user_credentials<'a>(local_api: &'a ::factorio_mods_local::API, web_api: &'a ::factorio_mods_web::API) ->
	impl Future<Item = ::factorio_mods_common::UserCredentials, Error = ::Error> + 'a {

	use ::ResultExt;

	::async_block! {
		let mut existing_username = match local_api.user_credentials() {
			Ok(user_credentials) =>
				return Ok(user_credentials),

			Err(err) => {
				let existing_username = if let ::factorio_mods_local::ErrorKind::IncompleteUserCredentials(ref existing_username) = *err.kind() {
					Some(existing_username.clone())
				}
				else {
					None
				};

				if let Some(existing_username) = existing_username {
					existing_username
				}
				else {
					return Err(err).chain_err(|| "Could not read user credentials");
				}
			},
		};

		loop {
			println!("You need a Factorio account to download mods.");
			println!("Please provide your username and password to authenticate yourself.");

			let username = {
				let prompt: ::std::borrow::Cow<_> = existing_username.as_ref().map_or("Username: ".into(), |username| format!("Username [{}]: ", username).into());
				::rprompt::prompt_reply_stdout(&prompt).chain_err(|| "Could not read username")?
			};

			let username = match(username.is_empty(), existing_username) {
				(false, _) => ::factorio_mods_common::ServiceUsername(username),
				(true, Some(existing_username)) => existing_username,
				(true, None) => {
					existing_username = None;
					continue;
				},
			};

			let password = ::rpassword::prompt_password_stdout("Password (not shown): ").chain_err(|| "Could not read password")?;

			match ::await!(web_api.login(username.clone(), &password)) {
				Ok(user_credentials) => {
					println!("Logged in successfully.");
					local_api.save_user_credentials(user_credentials.clone()).chain_err(|| "Could not save player-data.json")?;
					return Ok(user_credentials);
				},

				Err(::factorio_mods_web::Error(::factorio_mods_web::ErrorKind::LoginFailure(message), _)) => {
					println!("Authentication error: {}", message);
					existing_username = Some(username);
				},

				Err(err) =>
					return Err(err).chain_err(|| "Authentication error"),
			}
		}
	}
}

pub fn prompt_continue() -> ::Result<bool> {
	use ::ResultExt;

	loop {
		let choice = ::rprompt::prompt_reply_stdout("Continue? [y/n]: ").chain_err(|| "Could not read continue response")?;
		match &*choice {
			"y" | "Y" => return Ok(true),
			"n" | "N" => return Ok(false),
			_ => continue,
		}
	}
}
