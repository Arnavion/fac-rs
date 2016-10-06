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

			println!("Name: {}", mod_.name.0);
			println!("Author: {}", mod_.owner.0.join(", "));
			println!("Title: {}", mod_.title.0);
			println!("Summary: {}", mod_.summary.0);
			println!("Description:");
			for line in mod_.description.0.lines() {
				println!("    {}", line);
			}

			println!("Tags: {}", ::itertools::join(mod_.tags.iter().map(|t| &t.name.0), ", "));

			if !mod_.homepage.0.is_empty() {
				println!("Homepage: {}", mod_.homepage.0);
			}

			if !mod_.github_path.0.is_empty() {
				println!("GitHub page: https://github.com/{}", mod_.github_path.0);
			}

			// println!("License: {}", mod_.license_name.0);

			println!("Game versions: {}", ::itertools::join(mod_.game_versions.iter().map(|v| &v.0), ", "));

			println!("Releases:");
			if mod_.releases.is_empty() {
				println!("    No releases");
			}
			else {
				for release in mod_.releases {
					println!("    Version: {:-9} Game version: {:-9}", release.version.0, release.factorio_version.0);
				}
			}

			println!("");
		}
	}
}
