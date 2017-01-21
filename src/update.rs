pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Update installed mods."))
	}

	fn run<'a>(&self, _: &::clap::ArgMatches<'a>, local_api: ::Result<::factorio_mods_local::API>, web_api: ::Result<::factorio_mods_web::API>) -> ::Result<()> {
		let local_api = local_api?;
		let web_api = web_api?;

		let config = ::config::Config::load(&local_api)?;
		::solve::compute_and_apply_diff(&local_api, &web_api, config.mods())?;

		Ok(())
	}
}
