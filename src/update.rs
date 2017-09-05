use ::futures::{ future, Future };

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Update installed mods."))
	}

	fn run<'a>(
		&'a self,
		_: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		let (local_api, web_api) = match (local_api, web_api) {
			(Ok(local_api), Ok(web_api)) => (local_api, web_api),
			(Err(err), _) | (_, Err(err)) => return Box::new(future::err(err)),
		};

		let config = match ::config::Config::load(local_api) {
			Ok(config) => config,
			Err(err) => return Box::new(future::err(err)),
		};

		Box::new(
			::solve::compute_and_apply_diff(local_api, web_api, config.mods)
			.map(|_| ()))
	}
}
