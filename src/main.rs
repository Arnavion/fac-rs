//! A CLI tool to manage Factorio mods.

#![deny(rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::indexing_slicing,
	clippy::large_enum_variant,
	clippy::similar_names,
	clippy::type_complexity,
	clippy::use_self,
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

use failure::Fail;

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
async fn main() -> Result<(), DisplayableError> {
	use failure::ResultExt;

	std::env::set_var("RUST_BACKTRACE", "1");

	let options: Options = structopt::StructOpt::from_args();

	let client = if let Some(proxy_url) = options.proxy {
		let builder = crate::reqwest::ClientBuilder::new();
		let builder = builder.proxy(reqwest::Proxy::all(&proxy_url).context("Couldn't parse proxy URL")?);
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

	let local_api: Result<_, failure::Error> = match (&config.install_directory, &config.user_directory) {
		(Some(install_directory), Some(user_directory)) =>
			factorio_mods_local::API::new(install_directory, user_directory)
			.context("Could not initialize local API").map_err(Into::into),

		(None, _) => Err(
			factorio_mods_local::Error::from(factorio_mods_local::ErrorKind::InstallDirectoryNotFound)
			.context(r#"Could not initialize local API. Consider setting "install_directory" to the path in the config file."#).into()),

		(_, None) => Err(
			factorio_mods_local::Error::from(factorio_mods_local::ErrorKind::UserDirectoryNotFound)
			.context(r#"Could not initialize local API. Consider setting "user_directory" to the path in the config file."#).into()),
	};

	if config.mods.is_none() {
		if let Ok(local_api) = &local_api {
			// Default mods list is the list of all currently installed mods with a * requirement
			let installed_mods: Result<_, failure::Error> =
				local_api.installed_mods().context("Could not enumerate installed mods")?
				.map(|mod_| Ok(
					mod_
					.map(|mod_| (mod_.info.name, factorio_mods_common::ModVersionReq(semver::VersionReq::any())))
					.context("Could not process an installed mod")?))
				.collect();
			let mods = installed_mods.context("Could not enumerate installed mods")?;
			config.mods = Some(mods);
		}
	}

	let web_api = factorio_mods_web::API::new(client).context("Could not initialize web API").map_err(Into::into);


	match options.subcommand {
		SubCommand::Disable(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			prompt_override,
		).await?,

		SubCommand::Enable(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			prompt_override,
		).await?,

		SubCommand::Install(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			config,
			prompt_override,
		).await?,

		SubCommand::List(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
		).await?,

		SubCommand::Remove(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			config,
			prompt_override,
		).await?,

		SubCommand::Search(parameters) => parameters.run(
			match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
		).await?,

		SubCommand::Show(parameters) => parameters.run(
			match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
		).await?,

		SubCommand::Update(parameters) => parameters.run(
			match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			config,
			prompt_override,
		).await?,
	}

	Ok(())
}

struct DisplayableError(failure::Error);

impl std::fmt::Debug for DisplayableError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{}", self.0.as_fail())?;

		for fail in self.0.iter_causes() {
			writeln!(f)?;
			writeln!(f, "Caused by: {}", fail)?;
		}

		writeln!(f)?;
		writeln!(f, "{}", self.0.backtrace())?;

		Ok(())
	}
}

impl<T> From<T> for DisplayableError where T: Into<failure::Error> {
	fn from(err: T) -> Self {
		DisplayableError(err.into())
	}
}
