pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Remove mods.")
			(@arg mods: ... +required index(1) "mod names to remove"))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: ::Result<::factorio_mods_local::API>, web_api: ::Result<::factorio_mods_web::API>) -> ::Result<()> {
		let local_api = local_api?;
		let web_api = web_api?;

		let mods = matches.values_of("mods").unwrap();

		let config = ::config::Config::load(&local_api)?;
		let mut reqs = config.mods().clone();
		for mod_ in mods {
			let name = ::factorio_mods_common::ModName::new(mod_.to_string());
			reqs.remove(&name);
		}

		if ::solve::compute_and_apply_diff(&local_api, &web_api, &reqs)? {
			let config = config.with_mods(reqs);
			config.save()?;
		}

		Ok(())
	}
}