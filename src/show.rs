pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Show details about specific mods.")
			(@arg mods: ... +required index(1) "mods to show"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, _: ::Result<::factorio_mods_local::API>, web_api: ::Result<::factorio_mods_web::API>) -> ::Result<()> {
		use ::ResultExt;

		let web_api = web_api?;

		let names = matches.values_of("mods").unwrap();

		for name in names {
			let mod_ = web_api.get(&::factorio_mods_common::ModName::new(name.to_string())).chain_err(|| format!("Could not retrieve mod {}", name))?;

			println!("Name: {}", mod_.name());
			println!("Author: {}", ::itertools::join(mod_.owner(), ", "));
			println!("Title: {}", mod_.title());
			println!("Summary: {}", mod_.summary());
			println!("Description:");
			for line in mod_.description().lines() {
				println!("    {}", line);
			}

			println!("Tags: {}", ::itertools::join(mod_.tags().iter().map(|t| t.name()), ", "));

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

		Ok(())
	}
}
