use ::futures::{ future, Future, IntoFuture };

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

	match local_api.user_credentials() {
		Ok(user_credentials) =>
			future::Either::A(future::ok(user_credentials)),

		Err(err) => {
			let existing_username = if let ::factorio_mods_local::ErrorKind::IncompleteUserCredentials(ref existing_username) = *err.kind() {
				Some(existing_username.clone())
			}
			else {
				None
			};

			let existing_username = if let Some(existing_username) = existing_username {
				existing_username
			}
			else {
				return future::Either::A(Err(err).chain_err(|| "Could not read user credentials").into_future());
			};

			future::Either::B(
				future::loop_fn((), move |()| {
					println!("You need a Factorio account to download mods.");
					println!("Please provide your username and password to authenticate yourself.");
					match existing_username {
						Some(ref username) => print!("Username [{}]: ", username),
						None => print!("Username: "),
					}
					let mut stdout = ::std::io::stdout();
					if let Err(err) = ::std::io::Write::flush(&mut stdout) {
						return future::Either::A(Err(err).chain_err(|| "Could not write to stdout").into_future());
					}

					let mut username = String::new();
					if let Err(err) = ::std::io::stdin().read_line(&mut username) {
						return future::Either::A(Err(err).chain_err(|| "Could not read from stdin").into_future());
					}

					let username = username.trim().to_string();
					let username = match(username.is_empty(), existing_username.as_ref()) {
						(false, _) => ::std::borrow::Cow::Owned(::factorio_mods_common::ServiceUsername::new(username)),
						(true, Some(username)) => ::std::borrow::Cow::Borrowed(username),
						_ => return future::Either::A(future::ok(future::Loop::Continue(()))),
					};

					let password = match ::rpassword::prompt_password_stdout("Password (not shown): ") {
						Ok(password) => password,
						Err(err) => return future::Either::A(Err(err).chain_err(|| "Could not read password").into_future()),
					};

					future::Either::B(
						web_api.login(username.into_owned(), &password)
						.then(move |user_credentials| match user_credentials {
							Ok(user_credentials) => {
								println!("Logged in successfully.");
								match local_api.save_user_credentials(&user_credentials) {
									Ok(()) =>
										Ok(future::Loop::Break(user_credentials)),

									Err(err) =>
										Err(err).chain_err(|| "Could not save player-data.json"),
								}
							},

							Err(err) =>
								Err(err).chain_err(|| "Authentication error")
						}))
				}))
		},
	}
}

pub fn prompt_continue() -> ::Result<bool> {
	use ::ResultExt;

	loop {
		let mut choice = String::new();

		print!("Continue? [y/n]: ");

		let mut stdout = ::std::io::stdout();
		::std::io::Write::flush(&mut stdout).chain_err(|| "Could not write to stdout")?;

		::std::io::stdin().read_line(&mut choice).chain_err(|| "Could not read from stdin")?;

		match choice.trim() {
			"y" | "Y" => return Ok(true),
			"n" | "N" => return Ok(false),
			_ => continue,
		}
	}
}
