pub struct SubCommand;

impl<FL, FW> ::util::SubCommand<FL, FW> for SubCommand {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a> {
		subcommand
			.about("Search the mods database.")
			.arg(
				::clap::Arg::with_name("query")
					.help("search string")
					.index(1))
	}

	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, _: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API {
		let web_api = web_api();

		let query = matches.value_of("query").unwrap_or("");

		let max_width = ::term_size::dimensions().map(|(w, _)| w);

		let iter = web_api.search(query, &[], None, None, None).unwrap();
		for mod_ in iter {
			let mod_ = mod_.unwrap();
			println!("{}", mod_.title());
			println!("    Name: {}", mod_.name());
			println!("    Tags: {}", mod_.tags());
			println!("");
			max_width.map_or_else(|| {
				println!("    {}", mod_.summary());
			}, |max_width| {
				::util::wrapping_println(mod_.summary(), "    ", max_width);
			});
			println!("");
		}
	}
}
