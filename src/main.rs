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

#[macro_use] extern crate clap;
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

fn main() -> Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	// Run everything in a separate thread because the default Windows main thread stack isn't big enough (1 MiB)
	std::thread::spawn(|| {
		let app = clap_app!(@app (app_from_crate!())
			(@setting SubcommandRequiredElseHelp)
			(@setting VersionlessSubcommands)
			(@arg config: -c --config +takes_value "Path to fac config file. Defaults to .../fac/config.json ")
			(@arg proxy: --proxy +takes_value "HTTP proxy URL")
			(@arg yes: -y --yes "Answer yes to all prompts")
			(@arg no: -n --no conflicts_with("yes") "Answer no to all prompts"));

		let app = app.subcommand(cache::build_subcommand(clap::SubCommand::with_name("cache")));
		let app = app.subcommand(enable_disable::build_disable_subcommand(clap::SubCommand::with_name("disable")));
		let app = app.subcommand(enable_disable::build_enable_subcommand(clap::SubCommand::with_name("enable")));
		let app = app.subcommand(install::build_subcommand(clap::SubCommand::with_name("install")));
		let app = app.subcommand(list::build_subcommand(clap::SubCommand::with_name("list")));
		let app = app.subcommand(remove::build_subcommand(clap::SubCommand::with_name("remove")));
		let app = app.subcommand(search::build_subcommand(clap::SubCommand::with_name("search")));
		let app = app.subcommand(show::build_subcommand(clap::SubCommand::with_name("show")));
		let app = app.subcommand(update::build_subcommand(clap::SubCommand::with_name("update")));

		let matches = app.get_matches();

		let config_file_path = matches.value_of_os("config").map(Into::into);

		let client = if let Some(proxy_url) = matches.value_of("proxy") {
			let builder = crate::reqwest::r#async::ClientBuilder::new();
			let builder = builder.proxy(reqwest::Proxy::all(proxy_url).chain_err(|| "Couldn't parse proxy URL")?);
			Some(builder)
		}
		else {
			None
		};

		let prompt_override = match (matches.is_present("yes"), matches.is_present("no")) {
			(true, false) => Some(true),
			(false, true) => Some(false),
			(false, false) => None,
			(true, true) => unreachable!(),
		};

		let (subcommand_name, subcommand_matches) = matches.subcommand();

		let local_api = factorio_mods_local::API::new().chain_err(|| "Could not initialize local API");
		let web_api = factorio_mods_web::API::new(client).chain_err(|| "Could not initialize web API");

		let matches = subcommand_matches.unwrap();

		let mut runtime = tokio::runtime::current_thread::Runtime::new().chain_err(|| "Could not start tokio runtime")?;

		match subcommand_name {
			"cache" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(cache::run(
				matches,
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				config_file_path,
			))))?,
			"disable" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(enable_disable::run_disable(
				matches,
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,
			"enable" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(enable_disable::run_enable(
				matches,
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				prompt_override,
			))))?,
			"install" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(install::run(
				matches,
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				config_file_path,
				prompt_override,
			))))?,
			"list" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(list::run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
			))))?,
			"remove" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(remove::run(
				matches,
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				config_file_path,
				prompt_override,
			))))?,
			"search" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(search::run(
				matches,
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,
			"show" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(show::run(
				matches,
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
			))))?,
			"update" => runtime.block_on(futures::TryFutureExt::compat(Box::pinned(update::run(
				match local_api { Ok(ref local_api) => Ok(local_api), Err(err) => Err(err) },
				match web_api { Ok(ref web_api) => Ok(web_api), Err(err) => Err(err) },
				config_file_path,
				prompt_override,
			))))?,
			_ => unreachable!(),
		}

		Ok(())
	}).join().unwrap()
}
