pub fn build_subcommand<'a>(subcommand: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
	clap_app!(@app (subcommand)
		(about: "Show details about specific mods.")
		(@arg mods: ... +required index(1) "mods to show"))
}

pub async fn run<'a>(
	matches: &'a clap::ArgMatches<'a>,
	web_api: crate::Result<&'a factorio_mods_web::API>,
) -> crate::Result<()> {
	use crate::ResultExt;

	let web_api = web_api?;

	let names = matches.values_of("mods").unwrap();
	let names = names.map(|name| factorio_mods_common::ModName(name.to_string()));

	let mut mods =
		futures::stream::futures_ordered(names.map(|name| async move {
			await!(web_api.get(&name))
			.chain_err(|| format!("Could not retrieve mod {}", name))
		}));
	let mut mods = std::pin::Pin::new(&mut mods);

	while let Some(mod_) = await!(futures::StreamExt::next(&mut *mods)) {
		let mod_ = mod_?;

		println!("Name: {}", mod_.name);
		println!("Author: {}", itertools::join(mod_.owner, ", "));
		println!("Title: {}", mod_.title);
		println!("Summary: {}", mod_.summary);

		let releases = mod_.releases;

		if releases.is_empty() {
			println!("Releases:");
			println!("    No releases");
		}
		else {
			let mut game_versions: std::collections::BTreeSet<_> = Default::default();
			for release in &releases {
				game_versions.insert(format!("{}", release.info_json.factorio_version));
			}
			println!("Game versions: {}", itertools::join(game_versions, ", "));

			println!("Releases:");
			for release in releases {
				println!("    Version: {:-9} Game version: {:-9}", release.version, release.info_json.factorio_version);
			}
		}

		println!();
	}

	Ok(())
}
