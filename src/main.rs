//! A CLI tool to manage Factorio mods.

#[macro_use]
extern crate clap;
extern crate factorio_mods_common;
extern crate factorio_mods_local;
extern crate factorio_mods_web;
extern crate hyper;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rpassword;
extern crate semver;
extern crate term_size;
extern crate unicode_segmentation;

mod install;
mod list;
mod search;
mod show;

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
		(@setting VersionlessSubcommands))
		.arg(
			::clap::Arg::with_name("proxy-hostname")
				.long("proxy-hostname")
				.takes_value(true)
				.help("HTTP proxy hostname")
				.requires("proxy-port"))
		.arg(
			::clap::Arg::with_name("proxy-port")
				.long("proxy-port")
				.takes_value(true)
				.help("HTTP proxy port")
				.requires("proxy-hostname"));

	let app = subcommands.iter().fold(app, |app, (name, subcommand)| {
		app.subcommand(
			subcommand.build_subcommand(
				clap::SubCommand::with_name(name)))
	});

	let matches = app.get_matches();

	let client = match (matches.value_of("proxy-hostname"), matches.value_of("proxy-port")) {
		(Some(proxy_hostname), Some(proxy_port)) => Some(::hyper::Client::with_http_proxy(proxy_hostname.to_string(), proxy_port.parse().unwrap())),
		_ => None,
	};

	let subcommand_name = matches.subcommand_name().unwrap();
	let subcommand = subcommands[subcommand_name];

	subcommand.run(
		matches.subcommand_matches(subcommand_name).unwrap(),
		|| factorio_mods_local::API::new().unwrap(),
		|| factorio_mods_web::API::new(None, None, client).unwrap());
}
