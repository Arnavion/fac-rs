use ::futures::{ future, Future, Stream };

pub struct SubCommand;

impl ::util::SubCommand for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		clap_app!(@app (subcommand)
			(about: "Search the mods database.")
			(@arg query: index(1) "search string"))
	}

	fn run<'a>(
		&'a self,
		matches: &'a ::clap::ArgMatches<'a>,
		_: ::Result<&'a ::factorio_mods_local::API>,
		web_api: ::Result<&'a ::factorio_mods_web::API>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		use ::ResultExt;

		let web_api = match web_api {
			Ok(web_api) => web_api,
			Err(err) => return Box::new(future::err(err)),
		};

		let query = matches.value_of("query").unwrap_or("");

		Box::new(
			web_api.search(query, &[], None, None, None)
			.for_each(|mod_| {
				println!("{}", mod_.title());
				println!("    Name: {}", mod_.name());
				println!("    Tags: {}", ::itertools::join(mod_.tags().iter().map(|t| t.name()), ", "));
				println!("");
				::util::wrapping_println(mod_.summary(), "    ");
				println!("");

				Ok(())
			})
			.or_else(|err| Err(err).chain_err(|| "Could not retrieve mods")))
	}
}
