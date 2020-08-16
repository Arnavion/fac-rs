//! A CLI tool to manage Factorio mods.

#![deny(rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::similar_names,
	clippy::too_many_lines,
	clippy::type_complexity,
)]

mod enable_disable;
mod install;
mod list;
mod remove;
mod search;
mod show;
mod update;

mod config;
mod solve;
mod util;

use factorio_mods_web::reqwest;

#[derive(Debug, structopt::StructOpt)]
#[structopt(about, author)]
#[structopt(setting(structopt::clap::AppSettings::VersionlessSubcommands))]
pub(crate) struct Options {
	#[structopt(help = "Path to fac config file. Defaults to .../fac/config.json", short = "c", parse(from_os_str))]
	config: Option<std::path::PathBuf>,

	#[structopt(help = "HTTP proxy URL")]
	proxy: Option<String>,

	#[structopt(help = "Answer yes to all prompts", short = "y")]
	yes: bool,

	#[structopt(help = "Answer no to all prompts", short = "n", conflicts_with = "yes")]
	no: bool,

	#[structopt(subcommand)]
	subcommand: SubCommand,
}

#[derive(Debug, structopt::StructOpt)]
pub(crate) enum SubCommand {
	#[structopt(name = "disable", about = "Disable mods")]
	Disable(enable_disable::DisableSubCommand),

	#[structopt(name = "enable", about = "Enable mods")]
	Enable(enable_disable::EnableSubCommand),

	#[structopt(name = "install", about = "Install (or update) mods")]
	Install(install::SubCommand),

	#[structopt(name = "list", about = "List installed mods and their status")]
	List(list::SubCommand),

	#[structopt(name = "remove", about = "Remove mods")]
	Remove(remove::SubCommand),

	#[structopt(name = "search", about = "Search the mods database")]
	Search(search::SubCommand),

	#[structopt(name = "show", about = "Show details about specific mods")]
	Show(show::SubCommand),

	#[structopt(name = "update", about = "Update installed mods")]
	Update(update::SubCommand),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let options: Options = structopt::StructOpt::from_args();

	let client = if let Some(proxy_url) = options.proxy {
		let builder = crate::reqwest::ClientBuilder::new();
		let builder = builder.proxy(reqwest::Proxy::all(&proxy_url).context("couldn't parse proxy URL")?);
		Some(builder)
	}
	else {
		None
	};

	let prompt_override = match (options.yes, options.no) {
		(true, false) => Some(true),
		(false, true) => Some(false),
		(false, false) => None,
		(true, true) => unreachable!(),
	};

	let mut config = crate::config::Config::load(options.config)?;

	let local_api: Result<_, crate::Error> = match (&config.install_directory, &config.user_directory) {
		(Some(install_directory), Some(user_directory)) =>
			factorio_mods_local::API::new(install_directory, user_directory)
			.context("could not initialize local API").map_err(Into::into),

		(None, _) => Err(
			factorio_mods_local::Error::from(factorio_mods_local::ErrorKind::InstallDirectoryNotFound)
			.context(r#"could not initialize local API. Consider setting "install_directory" to the path in the config file."#)),

		(_, None) => Err(
			factorio_mods_local::Error::from(factorio_mods_local::ErrorKind::UserDirectoryNotFound)
			.context(r#"could not initialize local API. Consider setting "user_directory" to the path in the config file."#)),
	};

	if config.mods.is_none() {
		if let Ok(local_api) = &local_api {
			// Default mods list is the list of all currently installed mods with a * requirement
			let installed_mods =
				itertools::Itertools::try_collect::<_, _, _>(
					local_api.installed_mods().context("could not enumerate installed mods")?
					.map(|mod_|
						mod_
						.map(|mod_| (mod_.info.name, factorio_mods_common::ModVersionReq(semver::VersionReq::any())))
						.context("could not process an installed mod")))
				.context("could not enumerate installed mods")?;
			config.mods = Some(installed_mods);
		}
	}

	let web_api = factorio_mods_web::API::new(client).context("could not initialize web API");


	match options.subcommand {
		SubCommand::Disable(parameters) => parameters.run(
			&local_api?,
			prompt_override,
		).await?,

		SubCommand::Enable(parameters) => parameters.run(
			&local_api?,
			prompt_override,
		).await?,

		SubCommand::Install(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,

		SubCommand::List(parameters) => parameters.run(
			&local_api?,
		).await?,

		SubCommand::Remove(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,

		SubCommand::Search(parameters) => parameters.run(
			&web_api?,
		).await?,

		SubCommand::Show(parameters) => parameters.run(
			&web_api?,
		).await?,

		SubCommand::Update(parameters) => parameters.run(
			&local_api?,
			&web_api?,
			config,
			prompt_override,
		).await?,
	}

	Ok(())
}

struct Error(Box<dyn std::error::Error>, backtrace::Backtrace);

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{}", self.0)?;

		let mut source = self.0.source();
		while let Some(err) = source {
			writeln!(f, "caused by: {}", err)?;
			source = err.source();
		}

		writeln!(f)?;
		writeln!(f, "{:?}", self.1)?;

		Ok(())
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {
}

impl From<&'_ str> for Error {
	fn from(err: &str) -> Self {
		Error(err.into(), Default::default())
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self {
		Error(err.into(), Default::default())
	}
}

trait ErrorExt: std::error::Error + Sized + 'static {
	fn context<D>(self, context: D) -> Error where D: std::fmt::Display + std::fmt::Debug + 'static {
		#[derive(Debug)]
		struct ErrorWithContext<D, E> {
			context: D,
			err: E,
		}

		impl<D, E> std::fmt::Display for ErrorWithContext<D, E> where D: std::fmt::Display, E: std::error::Error {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				self.context.fmt(f)
			}
		}

		impl<D, E> std::error::Error for ErrorWithContext<D, E> where D: std::fmt::Display + std::fmt::Debug, E: std::error::Error + 'static {
			fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
				Some(&self.err)
			}
		}

		Error(Box::new(ErrorWithContext { context, err: self }), Default::default())
	}
}

impl<E> ErrorExt for E where E: std::error::Error + 'static {
}

trait ResultExt<T> {
	fn context<D>(self, context: D) -> Result<T, Error> where D: std::fmt::Display + std::fmt::Debug + 'static;

	fn with_context<F, D>(self, context: F) -> Result<T, Error> where F: FnOnce() -> D, D: std::fmt::Display + std::fmt::Debug + 'static;
}

impl<T, E> ResultExt<T> for Result<T, E> where E: ErrorExt {
	fn context<D>(self, context: D) -> Result<T, Error> where D: std::fmt::Display + std::fmt::Debug + 'static {
		self.map_err(|err| err.context(context))
	}

	fn with_context<F, D>(self, context: F) -> Result<T, Error> where F: FnOnce() -> D, D: std::fmt::Display + std::fmt::Debug + 'static {
		self.map_err(|err| err.context(context()))
	}
}
