#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
	#[structopt(help = "mods to show", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		web_api: Result<&'a factorio_mods_web::API, failure::Error>,
	) -> Result<(), failure::Error> {
		use failure::ResultExt;

		let web_api = web_api?;

		let mut mods =
			futures::stream::futures_ordered(self.names.into_iter().map(|name| async move {
				await!(web_api.get(&name))
				.with_context(|_| format!("Could not retrieve mod {}", name))
			}));

		while let Some(mod_) = await!(futures::StreamExt::next(&mut mods)) {
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
}
