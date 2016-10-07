use util;

pub struct SubCommand;

impl util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("Show details about specific mods.")
			.arg(
				::clap::Arg::with_name("mods")
					.help("mods to show")
					.index(1)
					.multiple(true)
					.required(true))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, api: ::factorio_mods_api::API) {
		let names = matches.values_of("mods").unwrap();

		for name in names {
			let mod_ = api.get(::factorio_mods_api::ModName(name.to_string())).unwrap();

			println!("Name: {}", mod_.name);
			println!("Author: {}", mod_.owner);
			println!("Title: {}", mod_.title);
			println!("Summary: {}", mod_.summary);
			println!("Description:");
			for line in mod_.description.0.lines() {
				println!("    {}", line);
			}

			println!("Tags: {}", mod_.tags);

			if !mod_.homepage.0.is_empty() {
				println!("Homepage: {}", mod_.homepage);
			}

			if !mod_.github_path.0.is_empty() {
				println!("GitHub page: https://github.com/{}", mod_.github_path);
			}

			println!("License: {}", mod_.license_name);

			println!("Game versions: {}", ::itertools::join(mod_.game_versions.iter(), ", "));

			println!("Releases:");
			if mod_.releases.is_empty() {
				println!("    No releases");
			}
			else {
				for release in mod_.releases {
					println!("    Version: {:-9} Game version: {:-9}", release.version, release.factorio_version);
				}
			}

			println!("");
		}
	}
}
