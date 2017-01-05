//! A CLI tool to manage Factorio mods.

#![feature(ordering_chaining, proc_macro)]

extern crate appdirs;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_struct;
extern crate factorio_mods_common;
extern crate factorio_mods_local;
extern crate factorio_mods_web;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rpassword;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_size;
extern crate unicode_segmentation;

mod install;
mod list;
mod search;
mod show;

mod config;
mod util;

fn main() {
	let install_subcommand = install::SubCommand;
	let list_subcommand = list::SubCommand;
	let search_subcommand = search::SubCommand;
	let show_subcommand = show::SubCommand;
	let mut subcommands = std::collections::HashMap::<_, &util::SubCommand<_, _>>::new();
	subcommands.insert("install", &install_subcommand);
	subcommands.insert("list", &list_subcommand);
	subcommands.insert("search", &search_subcommand);
	subcommands.insert("show", &show_subcommand);
	let subcommands = subcommands;

	let app = clap_app!(fac =>
		(author: crate_authors!())
		(version: crate_version!())
		(about: "fac")
		(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands));

	let app = subcommands.iter().fold(app, |app, (name, subcommand)| {
		app.subcommand(
			subcommand.build_subcommand(
				clap::SubCommand::with_name(name)))
	});

	let matches = app.get_matches();

	let subcommand_name = matches.subcommand_name().unwrap();
	let subcommand = subcommands[subcommand_name];

	subcommand.run(
		matches.subcommand_matches(subcommand_name).unwrap(),
		|| factorio_mods_local::API::new().unwrap(),
		|| factorio_mods_web::API::new(None, None, None).unwrap());
}
