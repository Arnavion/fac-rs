pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Update installed mods."))
	}

	fn run<'a>(&self, _: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();
		let local_api = local_api();

		let config = ::config::Config::load(&local_api);
		::solve::compute_and_apply_diff(&local_api, &web_api, config.mods());
	}
}
