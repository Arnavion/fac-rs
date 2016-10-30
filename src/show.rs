pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
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

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, _: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();

		let names = matches.values_of("mods").unwrap();

		for name in names {
			let mod_ = web_api.get(::factorio_mods_common::ModName::new(name.to_string())).unwrap();

			println!("Name: {}", mod_.name());
			println!("Author: {}", mod_.owner());
			println!("Title: {}", mod_.title());
			println!("Summary: {}", mod_.summary());
			println!("Description:");
			for line in mod_.description().lines() {
				println!("    {}", line);
			}

			println!("Tags: {}", mod_.tags());

			let homepage = mod_.homepage();
			if !homepage.is_empty() {
				println!("Homepage: {}", homepage);
			}

			let github_path = mod_.github_path();
			if !github_path.is_empty() {
				println!("GitHub page: https://github.com/{}", github_path);
			}

			println!("License: {}", mod_.license_name());

			println!("Game versions: {}", ::itertools::join(mod_.game_versions(), ", "));

			println!("Releases:");
			let releases = mod_.releases();
			if releases.is_empty() {
				println!("    No releases");
			}
			else {
				for release in releases {
					println!("    Version: {:-9} Game version: {:-9}", release.version(), release.factorio_version());
				}
			}

			println!("");
		}
	}
}
