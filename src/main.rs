//! A CLI tool to manage Factorio mods.

#![feature(
	arbitrary_self_types,
	async_await,
	await_macro,
	futures_api,
	nll,
	pin,
)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::default_trait_access,
	clippy::indexing_slicing,
	clippy::large_enum_variant,
	clippy::similar_names,
	clippy::type_complexity,
	clippy::use_self,
)]

#[macro_use] extern crate lazy_static;

use factorio_mods_web::reqwest;

mod cache;
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

#[derive(Debug, derive_error_chain::ErrorChain)]
pub enum ErrorKind {
	Msg(String),
}

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
	#[structopt(name = "cache", about = "Manage the local mods cache.")]
	Cache(cache::SubCommand),

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

fn main() -> Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	// Run everything in a separate thread because the default Windows main thread stack isn't big enough (1 MiB)
	std::thread::spawn(|| {
		let options: Options = structopt::StructOpt::from_args();

		let client = if let Some(proxy_url) = options.proxy {
			let builder = crate::reqwest::r#async::ClientBuilder::new();
			let builder = builder.proxy(reqwest::Proxy::all(&proxy_url).chain_err(|| "Couldn't parse proxy URL")?);
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

		let local_api = factorio_mods_local::API::new().chain_err(|| "Could not initialize local API");
		let web_api = factorio_mods_web::API::new(client).chain_err(|| "Could not initialize web API");

		let mut runtime = tokio::runtime::current_thread::Runtime::new().chain_err(|| "Could not start tokio runtime")?;

		match options.subcommand {
			SubCommand::Cache(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				options.config,
			))))?,

			SubCommand::Disable(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,

			SubCommand::Enable(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,

			SubCommand::Install(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,

			SubCommand::List(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Remove(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,

			SubCommand::Search(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Show(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,

			SubCommand::Update(parameters) => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(parameters.run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				options.config,
				prompt_override,
			))))?,
		}

		Ok(())
	}).join().unwrap()
}
