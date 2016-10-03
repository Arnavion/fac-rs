#[macro_use]
extern crate clap;

extern crate factorio_mods_api;

fn main() {
	let app =
		clap::App::new("fac-rs")
			.author(crate_authors!())
			.version(crate_version!())
			.about("fac-rs")
			.subcommand(
				clap::SubCommand::with_name("search")
					.about("Search the mods database.")
					.arg(
						clap::Arg::with_name("query")
							.help("search string")
							.index(1)
							.required(true)))
			.setting(clap::AppSettings::SubcommandRequiredElseHelp);

	let matches = app.get_matches();

	if let Some(ref matches) = matches.subcommand_matches("search") {
		let query = matches.value_of("query").unwrap();

		let api = factorio_mods_api::API::new(None, None, None).unwrap();

		let iter = api.search(query, vec![], None, None, None).unwrap();
		for mod_ in iter {
			match mod_ {
				Ok(mod_) => {
					println!("{}", mod_.title.0);
					println!("    Name: {}", mod_.name.0);
					println!("    Tags: {}", factorio_mods_api::DisplayableTags(mod_.tags));
					println!("");
					println!("    {}", mod_.summary.0);
					println!("");
				},
				Err(err) => {
					println!("{:?}", err);
					panic!(err)
				}
			}
		}
	}
}
