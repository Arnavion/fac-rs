use failure::{ Fail, ResultExt };

pub fn wrapping_println(s: &str, indent: &str) {
	#[allow(clippy::single_match_else)] // Bad clippy lint - false positive
	match term_size::dimensions() {
		Some((width, _)) => {
			let wrapper = textwrap::Wrapper {
				width,
				initial_indent: indent,
				subsequent_indent: indent,
				break_words: true,
				splitter: textwrap::NoHyphenation,
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

pub async fn ensure_user_credentials<'a>(
	local_api: &'a factorio_mods_local::API,
	web_api: &'a factorio_mods_web::API,
	prompt_override: Option<bool>,
) -> Result<factorio_mods_common::UserCredentials, failure::Error> {
	let mut existing_username = match local_api.user_credentials() {
		Ok(user_credentials) =>
			return Ok(user_credentials),

		Err(err) => if let factorio_mods_local::ErrorKind::IncompleteUserCredentials(existing_username) = err.kind() {
			existing_username.clone()
		}
		else {
			return Err(err.context("Could not read user credentials").into());
		},
	};

	loop {
		println!("You need a Factorio account to download mods.");
		println!("Please provide your username and password to authenticate yourself.");

		match prompt_override {
			Some(true) => failure::bail!("Exiting because --yes was specified ..."),
			Some(false) => failure::bail!("Exiting because --no was specified ..."),
			None => (),
		}

		let username = {
			let prompt: std::borrow::Cow<'_, _> = existing_username.as_ref().map_or("Username: ".into(), |username| format!("Username [{}]: ", username).into());
			rprompt::prompt_reply_stdout(&prompt).context("Could not read username")?
		};

		let username = match(username.is_empty(), existing_username) {
			(false, _) => factorio_mods_common::ServiceUsername(username),
			(true, Some(existing_username)) => existing_username,
			(true, None) => {
				existing_username = None;
				continue;
			},
		};

		let password = rpassword::prompt_password_stdout("Password (not shown): ").context("Could not read password")?;

		match web_api.login(username.clone(), &password).await {
			Ok(user_credentials) => {
				println!("Logged in successfully.");
				local_api.save_user_credentials(user_credentials.clone()).context("Could not save player-data.json")?;
				return Ok(user_credentials);
			},

			Err(err) => match err.kind() {
				factorio_mods_web::ErrorKind::LoginFailure(message) => {
					println!("Authentication error: {}", message);
					existing_username = Some(username);
				},

				_ =>
					return Err(err.context("Authentication error").into()),
			},
		}
	}
}

pub fn prompt_continue(prompt_override: Option<bool>) -> Result<bool, failure::Error> {
	match prompt_override {
		Some(true) => {
			println!("Continue? [y/n]: y");
			Ok(true)
		},

		Some(false) => {
			println!("Continue? [y/n]: n");
			Ok(false)
		},

		None => loop {
			let choice = rprompt::prompt_reply_stdout("Continue? [y/n]: ").context("Could not read continue response")?;
			match &*choice {
				"y" | "Y" => return Ok(true),
				"n" | "N" => return Ok(false),
				_ => continue,
			}
		},
	}

}
