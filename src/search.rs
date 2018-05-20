use ::futures::Future;

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
		_: Option<bool>,
	) -> Box<Future<Item = (), Error = ::Error> + 'a> {
		use ::ResultExt;

		#[cfg_attr(feature = "cargo-clippy", allow(unit_arg))]
		Box::new(::async_block! {
			let query = matches.value_of("query").unwrap_or("");

			let web_api = web_api?;

			let r: Result<_, ::factorio_mods_web::Error> = do catch {
				#[async] for mod_ in web_api.search(query) {
					println!("{}", mod_.title);
					println!("    Name: {}", mod_.name);
					println!();
					::util::wrapping_println(&mod_.summary.0, "    ");
					println!();
				}
			};

			r.chain_err(|| "Could not retrieve mods")
		})
	}
}
