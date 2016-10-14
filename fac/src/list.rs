pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("List installed mods and their status.")
	}

	fn run<'a>(&self, _: &::clap::ArgMatches<'a>, _: ::factorio_mods_api::API, manager: ::factorio_mods_local::Manager) {
		let mut installed_mods: Vec<_> = manager.installed_mods().unwrap().map(|m| m.unwrap()).collect();
		if installed_mods.is_empty() {
			println!("No installed mods.");
			return;
		}

		installed_mods.sort_by(|m1, m2| {
			match m1.enabled().cmp(m2.enabled()) {
				::std::cmp::Ordering::Equal => m1.name().cmp(m2.name()),
				o => o.reverse(),
			}
		});
		let installed_mods = installed_mods;

		println!("Installed mods:");

		for installed_mod in installed_mods {
			let mut tags: Vec<&'static str> = vec![];
			if !installed_mod.enabled() {
				tags.push("disabled");
			}
			if let ::factorio_mods_local::InstalledMod::Unpacked { .. } = installed_mod {
				tags.push("unpacked");
			}

			let tags_string = if !tags.is_empty() { format!(" ({})", tags.join(", ")) } else { "".to_string() };

			println!("    {} {}{}", installed_mod.name(), installed_mod.version(), tags_string);
		}
	}
}
