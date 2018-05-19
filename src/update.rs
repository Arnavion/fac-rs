use ::futures::Future;

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
		Box::new(::async_block! {
			let local_api = local_api?;
			let web_api = web_api?;

			let config = ::config::Config::load(local_api)?;

			::await!(::solve::compute_and_apply_diff(local_api, web_api, config))?;

			Ok(())
		})
	}
}
