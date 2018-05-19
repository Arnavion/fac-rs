use ::futures::Future;

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Remove mods.")
			(@arg mods: ... +required index(1) "mod names to remove"))
	}

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		local_api: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		Box::new(::async_block! {
			let mods = matches.values_of("mods").unwrap();

			let local_api = local_api?;
			let web_api = web_api?;

			let mut config = ::config::Config::load(local_api)?;

			for mod_ in mods {
				let name = ::factorio_mods_common::ModName(mod_.to_string());
				config.mods.remove(&name);
			}

			::await!(::solve::compute_and_apply_diff(local_api, web_api, config))?;

			Ok(())
		})
	}
}
