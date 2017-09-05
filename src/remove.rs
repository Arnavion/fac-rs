use ::futures::{ future, Future };

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
		let mods = matches.values_of("mods").unwrap();

		let (local_api, web_api) = match (local_api, web_api) {
			(Ok(local_api), Ok(web_api)) => (local_api, web_api),
			(Err(err), _) | (_, Err(err)) => return Box::new(future::err(err)),
		};

		let mut config = match ::config::Config::load(local_api) {
			Ok(config) => config,
			Err(err) => return Box::new(future::err(err)),
		};

		let mut reqs = ::std::mem::replace(&mut config.mods, Default::default());
		for mod_ in mods {
			let name = ::factorio_mods_common::ModName::new(mod_.to_string());
			reqs.remove(&name);
		}

		Box::new(
			::solve::compute_and_apply_diff(local_api, web_api, reqs)
			.and_then(move |(result, reqs)| Ok(if result {
				config.mods = reqs;
				config.save()?
			})))
	}
}
