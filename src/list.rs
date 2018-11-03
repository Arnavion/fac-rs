#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, failure::Error>,
	) -> Result<(), failure::Error> {
		use failure::ResultExt;

		let local_api = local_api?;

		let mods_status = local_api.mods_status().context("Could not parse installed mods status")?;

		let installed_mods: Result<Vec<_>, _> =
			local_api.installed_mods()
			.context("Could not enumerate installed mods")?
			.map(|installed_mod| installed_mod.map(|installed_mod| {
				let enabled = mods_status.get(&installed_mod.info.name).cloned().unwrap_or(true);
				(installed_mod, enabled)
			}))
			.collect();
		let mut installed_mods = installed_mods.context("Could not enumerate installed mods")?;
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

				println!("    {} {}{}", installed_mod.0.info.name, installed_mod.0.info.version, tags_string);
			}
		}

		Ok(())
	}
}
