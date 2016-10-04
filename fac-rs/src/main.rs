#[macro_use]
extern crate clap;
extern crate factorio_mods_api;
extern crate term_size;
extern crate unicode_segmentation;

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

		let max_width = term_size::dimensions().map(|(w, _)| w);

		let iter = api.search(query, vec![], None, None, None).unwrap();
		for mod_ in iter {
			match mod_ {
				Ok(mod_) => {
					println!("{}", mod_.title.0);
					println!("    Name: {}", mod_.name.0);
					println!("    Tags: {}", factorio_mods_api::DisplayableTags(&mod_.tags));
					println!("");
					max_width.map_or_else(|| {
						println!("    {}", mod_.summary.0);
					}, |max_width| {
						wrapping_println(mod_.summary.0.as_str(), "    ", max_width);
					});
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

fn wrapping_println(s: &str, indent: &str, max_width: usize) {
	let max_len = max_width - indent.len();

	let graphemes: Vec<&str> = unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect();
	let mut graphemes = &graphemes[..];

	loop {
		print!("{}", indent);

		if graphemes.is_empty() {
			return;
		}

		if graphemes.len() <= max_len {
			for s in graphemes {
				print!("{}", s);
			}
			println!("");
			return;
		}

		let (line, remaining) = if let Some(last_space_pos) = graphemes[..max_len].iter().rposition(|&s| s == " ") {
			(&graphemes[..last_space_pos], &graphemes[last_space_pos + 1..])
		}
		else {
			(&graphemes[..max_len], &graphemes[max_len..])
		};

		for s in line {
			print!("{}", s);
		}
		println!("");

		graphemes = remaining;
	}
}
