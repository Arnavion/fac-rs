#[derive(clap::Parser)]
pub(crate) struct SubCommand {
}

impl SubCommand {
	pub(crate) fn run(
		local_api: &factorio_mods_local::Api,
	) -> anyhow::Result<()> {
		use anyhow::Context;

		let mods_status = local_api.mods_status().context("could not parse installed mods status")?;

		let mut installed_mods: Vec<_> =
			itertools::Itertools::try_collect(
				local_api.installed_mods()
				.context("could not enumerate installed mods")?
				.map(|installed_mod| installed_mod.map(|installed_mod| {
					let enabled = mods_status.get(&installed_mod.info.name).copied().unwrap_or(true);
					(installed_mod, enabled)
				})))
			.context("could not enumerate installed mods")?;
		if installed_mods.is_empty() {
			println!("No installed mods.");
		}
		else {
			installed_mods.sort_by(|m1, m2|
				m1.1.cmp(&m2.1).reverse()
				.then_with(|| m1.0.info.name.cmp(&m2.0.info.name)));

			let installed_mods = installed_mods;

			println!("Installed mods:");

			for installed_mod in installed_mods {
				let mut tags = vec![];
				if !installed_mod.1 {
					tags.push("disabled");
				}
				if let factorio_mods_local::InstalledModType::Unpacked = installed_mod.0.mod_type {
					tags.push("unpacked");
				}

				let tags_string = if tags.is_empty() { String::new() } else { format!(" ({})", tags.join(", ")) };

				println!("    {} {}{tags_string}", installed_mod.0.info.name, installed_mod.0.info.version);
			}
		}

		Ok(())
	}
}
