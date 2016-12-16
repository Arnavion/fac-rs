pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "List installed mods and their status."))
	}

	fn run<'a>(&self, _: &::clap::ArgMatches<'a>, local_api: FL, _: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let local_api = local_api();

		let mut installed_mods: Vec<_> = local_api.installed_mods().unwrap().map(Result::unwrap).collect();
		if installed_mods.is_empty() {
			println!("No installed mods.");
			return;
		}

		installed_mods.sort_by(|m1, m2| {
			match m1.enabled().cmp(m2.enabled()) {
				::std::cmp::Ordering::Equal => m1.info().name().cmp(m2.info().name()),
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
			if let ::factorio_mods_local::InstalledModType::Unpacked = *installed_mod.mod_type() {
				tags.push("unpacked");
			}

			let tags_string = if !tags.is_empty() { format!(" ({})", tags.join(", ")) } else { String::new() };

			println!("    {} {}{}", installed_mod.info().name(), installed_mod.info().version(), tags_string);
		}
	}
}
