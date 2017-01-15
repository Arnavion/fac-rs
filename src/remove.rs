pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Remove mods.")
			(@arg mods: ... +required index(1) "mod names to remove"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();
		let local_api = local_api();

		let mods = matches.values_of("mods").unwrap();

		let config = ::config::Config::load(&local_api);
		let mut reqs = config.mods().clone();
		for mod_ in mods {
			let name = ::factorio_mods_common::ModName::new(mod_.to_string());
			reqs.remove(&name);
		}

		if ::solve::compute_and_apply_diff(&local_api, &web_api, &reqs) {
			let config = config.with_mods(reqs);
			config.save();
		}
	}
}
