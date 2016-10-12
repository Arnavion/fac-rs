#[macro_use]
extern crate clap;
extern crate factorio_mods_api;
extern crate factorio_mods_common;
extern crate factorio_mods_local;
extern crate itertools;
extern crate term_size;
extern crate unicode_segmentation;

mod list;
mod search;
mod show;

mod util;

fn main() {
	let list_subcommand = list::SubCommand;
	let search_subcommand = search::SubCommand;
	let show_subcommand = show::SubCommand;
	let mut subcommands = std::collections::HashMap::<&str, &util::SubCommand>::new();
	subcommands.insert("list", &list_subcommand);
	subcommands.insert("search", &search_subcommand);
	subcommands.insert("show", &show_subcommand);
	let subcommands = subcommands;

	let app = clap::App::new("fac")
		.author(crate_authors!())
		.version(crate_version!())
		.about("fac")
		.setting(clap::AppSettings::SubcommandRequiredElseHelp)
		.setting(clap::AppSettings::VersionlessSubcommands);

	let app = subcommands.iter().fold(app, |app, (name, subcommand)| {
		app.subcommand(
			subcommand.build_subcommand(
				clap::SubCommand::with_name(name)))
	});

	let matches = app.get_matches();
	let subcommand_name = matches.subcommand_name().unwrap();
	let subcommand = subcommands[subcommand_name];

	let api = factorio_mods_api::API::new(None, None, None).unwrap();
	let manager = factorio_mods_local::Manager { config: factorio_mods_local::Config::new().unwrap() };

	subcommand.run(matches.subcommand_matches(subcommand_name).unwrap(), api, manager);
}
