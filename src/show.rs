use ::futures::{ Future, stream };

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Show details about specific mods.")
			(@arg mods: ... +required index(1) "mods to show"))
	}

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		_: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		use ::ResultExt;

		Box::new(::async_block! {
			let web_api = web_api?;

			let names = matches.values_of("mods").unwrap();
			let names = names.into_iter().map(|name| ::factorio_mods_common::ModName::new(name.to_string()));

			let mods =
				stream::futures_ordered(names.map(|name| ::async_block! {
					::await!(web_api.get(&name))
					.chain_err(|| format!("Could not retrieve mod {}", name))
				}));

			#[async] for mod_ in mods {
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

				println!();
			}

			Ok(())
		})
	}
}
