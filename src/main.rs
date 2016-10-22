#[macro_use]
extern crate clap;
extern crate factorio_mods_api;
extern crate factorio_mods_common;
extern crate factorio_mods_local;
extern crate hyper;
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
		.setting(clap::AppSettings::VersionlessSubcommands)
		.arg(
			::clap::Arg::with_name("proxy-hostname")
				.long("proxy-hostname")
				.takes_value(true)
				.help("HTTP proxy hostname"))
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

	let api = factorio_mods_api::API::new(None, None, client).unwrap();
	let manager = factorio_mods_local::Manager::new().unwrap();

	let subcommand_name = matches.subcommand_name().unwrap();
	let subcommand = subcommands[subcommand_name];

	subcommand.run(matches.subcommand_matches(subcommand_name).unwrap(), api, manager);
}
