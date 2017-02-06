//! A CLI tool to manage Factorio mods.

#![feature(ordering_chaining)]

extern crate appdirs;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate derive_error_chain;
#[macro_use]
extern crate derive_struct;
extern crate factorio_mods_common;
extern crate factorio_mods_local;
extern crate factorio_mods_web;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate multimap;
extern crate petgraph;
extern crate regex;
extern crate rpassword;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_size;
extern crate unicode_segmentation;

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

#[derive(Debug, error_chain)]
pub enum ErrorKind {
	Msg(String),
}

quick_main!(|| -> Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let disable_subcommand = enable_disable::DisableSubCommand;
	let enable_subcommand = enable_disable::EnableSubCommand;
	let install_subcommand = install::SubCommand;
	let list_subcommand = list::SubCommand;
	let remove_subcommand = remove::SubCommand;
	let search_subcommand = search::SubCommand;
	let show_subcommand = show::SubCommand;
	let update_subcommand = update::SubCommand;
	let mut subcommands = std::collections::HashMap::<_, &util::SubCommand>::new();
	subcommands.insert("disable", &disable_subcommand);
	subcommands.insert("enable", &enable_subcommand);
	subcommands.insert("install", &install_subcommand);
	subcommands.insert("list", &list_subcommand);
	subcommands.insert("remove", &remove_subcommand);
	subcommands.insert("search", &search_subcommand);
	subcommands.insert("show", &show_subcommand);
	subcommands.insert("update", &update_subcommand);
	let subcommands = subcommands;

	let app = clap_app!(@app (app_from_crate!())
		(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands));

	let app = subcommands.iter().fold(app, |app, (name, subcommand)| {
		app.subcommand(
			subcommand.build_subcommand(
				clap::SubCommand::with_name(name)))
	});

	let matches = app.get_matches();

	let (subcommand_name, subcommand_matches) = matches.subcommand();
	let subcommand = subcommands[subcommand_name];

	subcommand.run(
		subcommand_matches.unwrap(),
		factorio_mods_local::API::new().chain_err(|| "Could not initialize local API"),
		factorio_mods_web::API::new(None).chain_err(|| "Could not initialize local API"))
});
