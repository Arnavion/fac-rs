#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
	#[structopt(help = "mods to show", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		web_api: &factorio_mods_web::Api,
	) -> Result<(), crate::Error> {
		use crate::ResultExt;

		let textwrap_options = crate::textwrap_options();

		let mut mods: futures_util::stream::FuturesOrdered<_> =
			self.names.into_iter().map(|name| async move {
				web_api.get(&name).await
				.with_context(|| format!("could not retrieve mod {}", name))
			}).collect();

		while let Some(mod_) = futures_util::TryStreamExt::try_next(&mut mods).await? {
			println!("Name: {}", mod_.name);
			println!("Author: {}", itertools::join(mod_.owner, ", "));
			println!("Title: {}", mod_.title);
			println!("Summary:");

			for line in mod_.summary.0.lines() {
				for line in textwrap::wrap(line, textwrap_options.clone()) {
					println!("{}", line);
				}
			}

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
					println!(
						"    Version: {:-9} Game version: {:-9}",
						format_args!("{}", release.version),
						format_args!("{}", release.info_json.factorio_version),
					);
				}
			}

			println!();
		}

		Ok(())
	}
}
