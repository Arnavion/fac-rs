use ::futures::{ Future, IntoFuture };

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "List installed mods and their status."))
	}

	fn run<'a>(
		&'a self,
		_: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		_: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		use ::ResultExt;

		let result: ::Result<_> = do catch {
			let local_api = local_api?;

			let installed_mods: Result<Vec<_>, _> = local_api.installed_mods().chain_err(|| "Could not enumerate installed mods")?.collect();
			let mut installed_mods = installed_mods.chain_err(|| "Could not enumerate installed mods")?;
			if installed_mods.is_empty() {
				println!("No installed mods.");
			}
			else {
				installed_mods.sort_by(|m1, m2|
					m1.enabled().cmp(&m2.enabled()).reverse()
					.then_with(|| m1.info().name().cmp(m2.info().name())));

				let installed_mods = installed_mods;

				println!("Installed mods:");

				for installed_mod in installed_mods {
					let mut tags = vec![];
					if !installed_mod.enabled() {
						tags.push("disabled");
					}
					if let ::factorio_mods_local::InstalledModType::Unpacked = installed_mod.mod_type() {
						tags.push("unpacked");
					}

					let tags_string = if tags.is_empty() { String::new() } else { format!(" ({})", tags.join(", ")) };

					println!("    {} {}{}", installed_mod.info().name(), installed_mod.info().version(), tags_string);
				}
			}
		};
		Box::new(result.into_future())
	}
}
