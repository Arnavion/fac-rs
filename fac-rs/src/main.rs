#[macro_use]
extern crate clap;
extern crate factorio_mods_api;
extern crate itertools;
extern crate term_size;
extern crate unicode_segmentation;

fn main() {
	let app =
		clap::App::new("fac")
			.author(crate_authors!())
			.version(crate_version!())
			.about("fac")
			.subcommand(
				clap::SubCommand::with_name("search")
					.about("Search the mods database.")
					.arg(
						clap::Arg::with_name("query")
							.help("search string")
							.index(1)
							.required(true)))
			.subcommand(
				clap::SubCommand::with_name("show")
					.about("Show details about specific mods.")
					.arg(
						clap::Arg::with_name("mods")
							.help("mods to show")
							.index(1)
							.multiple(true)
							.required(true)))
			.setting(clap::AppSettings::SubcommandRequiredElseHelp);

	let matches = app.get_matches();

	let max_width = term_size::dimensions().map(|(w, _)| w);

	let api = factorio_mods_api::API::new(None, None, None).unwrap();

	match matches.subcommand() {
		("search", Some(matches)) => {
			let query = matches.value_of("query").unwrap();

			let iter = api.search(query, &vec![], None, None, None).unwrap();
			for mod_ in iter {
				let mod_ = mod_.unwrap();
				println!("{}", mod_.title.0);
				println!("    Name: {}", mod_.name.0);
				println!("    Tags: {}", join(&mod_.tags, |t| &t.name.0));
				println!("");
				max_width.map_or_else(|| {
					println!("    {}", mod_.summary.0);
				}, |max_width| {
					wrapping_println(mod_.summary.0.as_str(), "    ", max_width);
				});
				println!("");
			}
		},

		("show", Some(matches)) => {
			let mods: Vec<_> = matches.values_of("mods").unwrap().collect();

			for mod_name in mods {
				let mod_ = api.get(factorio_mods_api::ModName(mod_name.to_string())).unwrap();

				println!("Name: {}", mod_.name.0);
				println!("Author: {}", mod_.owner.0.join(", "));
				println!("Title: {}", mod_.title.0);
				println!("Summary: {}", mod_.summary.0);
				println!("Description:");
				for line in mod_.description.0.lines() {
					println!("    {}", line);
				}

				println!("Tags: {}", join(&mod_.tags, |t| &t.name.0));

				if !mod_.homepage.0.is_empty() {
					println!("Homepage: {}", mod_.homepage.0);
				}

				if !mod_.github_path.0.is_empty() {
					println!("GitHub page: https://github.com/{}", mod_.github_path.0);
				}

				// println!("License: {}", mod_.license_name.0);

				println!("Game versions: {}", join(&mod_.game_versions, |v| &v.0));

				println!("Releases:");
				if mod_.releases.is_empty() {
					println!("    No releases");
				}
				else {
					for release in mod_.releases {
						println!("    Version: {:-9} Game version: {:-9}", release.version.0, release.factorio_version.0);
					}
				}

				println!("");
			}
		},

		_ => panic!(),
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

fn join<T, F>(v: &Vec<T>, f: F) -> String where F: FnMut(&T) -> &String {
	itertools::join(v.iter().map(f), ", ")
}
