use crate::{ ErrorExt, ResultExt };

pub(crate) async fn ensure_user_credentials(
	local_api: &factorio_mods_local::Api,
	web_api: &factorio_mods_web::Api,
	prompt_override: Option<bool>,
) -> Result<factorio_mods_common::UserCredentials, crate::Error> {
	let mut existing_username = match local_api.user_credentials() {
		Ok(user_credentials) =>
			return Ok(user_credentials),

		Err(factorio_mods_local::Error::IncompleteUserCredentials(existing_username)) =>
			existing_username.clone(),

		Err(err) =>
			return Err(err.context("could not read user credentials")),
	};

	loop {
		println!("You need a Factorio account to download mods.");
		println!("Please provide your username and password to authenticate yourself.");

		match prompt_override {
			Some(true) => return Err("Exiting because --yes was specified ...".into()),
			Some(false) => return Err("Exiting because --no was specified ...".into()),
			None => (),
		}

		let username = {
			let prompt: std::borrow::Cow<'_, _> =
				existing_username.as_ref().map_or_else(|| "Username: ".into(), |username| format!("Username [{}]: ", username).into());
			rprompt::prompt_reply_stdout(&prompt).context("could not read username")?
		};

		let username = match(username.is_empty(), existing_username) {
			(false, _) => factorio_mods_common::ServiceUsername(username),
			(true, Some(existing_username)) => existing_username,
			(true, None) => {
				existing_username = None;
				continue;
			},
		};

		let password = rpassword::prompt_password_stdout("Password (not shown): ").context("could not read password")?;

		match web_api.login(username.clone(), &password).await {
			Ok(user_credentials) => {
				println!("Logged in successfully.");
				local_api.save_user_credentials(user_credentials.clone()).context("could not save player-data.json")?;
				return Ok(user_credentials);
			},

			Err(factorio_mods_web::Error::LoginFailure(message)) => {
				println!("Authentication error: {}", message);
				existing_username = Some(username);
			},

			Err(err) => return Err(err.context("authentication error")),
		}
	}
}

pub(crate) fn prompt_continue(prompt_override: Option<bool>) -> Result<bool, crate::Error> {
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
			let choice = rprompt::prompt_reply_stdout("Continue? [y/n]: ").context("could not read continue response")?;
			match &*choice {
				"y" | "Y" => return Ok(true),
				"n" | "N" => return Ok(false),
				_ => continue,
			}
		},
	}

}
