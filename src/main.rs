//! A CLI tool to manage Factorio mods.

#![feature(
	async_await,
)]

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

use factorio_mods_web::reqwest;

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

#[derive(Debug, structopt_derive::StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::VersionlessSubcommands"))]
pub struct Options {
	#[structopt(help = "Path to fac config file. Defaults to .../fac/config.json", short = "c", parse(from_os_str))]
	pub config: Option<std::path::PathBuf>,

	#[structopt(help = "HTTP proxy URL")]
	pub proxy: Option<String>,

	#[structopt(help = "Answer yes to all prompts", short = "y")]
	pub yes: bool,

	#[structopt(help = "Answer no to all prompts", short = "n", conflicts_with = "yes")]
	pub no: bool,

	#[structopt(subcommand)]
	pub subcommand: SubCommand,
}

#[derive(Debug, structopt_derive::StructOpt)]
pub enum SubCommand {
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

fn main() -> Result<(), DisplayableError> {
	use failure::ResultExt;

	std::env::set_var("RUST_BACKTRACE", "1");

	// Run everything in a separate thread because the default Windows main thread stack isn't big enough (1 MiB)
	std::thread::spawn(|| {
		let options: Options = structopt::StructOpt::from_args();

		let client = if let Some(proxy_url) = options.proxy {
			let builder = crate::reqwest::r#async::ClientBuilder::new();
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

		let local_api = factorio_mods_local::API::new().context("Could not initialize local API").map_err(Into::into);
		let web_api = factorio_mods_web::API::new(client).context("Could not initialize web API").map_err(Into::into);

		let mut runtime = tokio::runtime::current_thread::Runtime::new().context("Could not start tokio runtime")?;

		match options.subcommand {
			SubCommand::Disable(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,

			SubCommand::Enable(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,

			SubCommand::Install(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,

			SubCommand::List(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Remove(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,

			SubCommand::Search(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Show(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Update(parameters) => runtime.block_on(futures_util::TryFutureExt::compat(Box::pin(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,
		}

		Ok(())
	}).join().unwrap().map_err(DisplayableError)
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
