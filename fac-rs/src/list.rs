use util;

pub struct SubCommand;

impl util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("List installed mods and their status.")
	}

	fn run<'a>(&self, _: &::clap::ArgMatches<'a>, _: ::factorio_mods_api::API, manager: ::factorio_mods_local::Manager) {
		let installed_mods: Vec<_> = manager.installed_mods().unwrap().collect();
		if installed_mods.is_empty() {
			println!("No installed mods.");
			return;
		}

		println!("Installed mods:");

		for installed_mod in installed_mods {
			let installed_mod = installed_mod.unwrap();

			println!("    {} {}{}", installed_mod.name(), installed_mod.version(), "");
		}
	}
}
