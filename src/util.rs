pub trait SubCommand<FL, FW> {
	fn build_subcommand<'a>(&self, subcommand: ::clap::App<'a, 'a>) -> ::clap::App<'a, 'a>;
	fn run<'a>(&self, matches: &::clap::ArgMatches<'a>, local_api: FL, web_api: FW)
		where FL: FnOnce() -> ::factorio_mods_local::API, FW: FnOnce() -> ::factorio_mods_web::API;
}

pub fn wrapping_println(s: &str, indent: &str, max_width: usize) {
	let max_len = max_width - indent.len();

	let graphemes: Vec<&str> = ::unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect();
	let mut graphemes = &graphemes[..];

	loop {
		if graphemes.is_empty() {
			return;
		}

		print!("{}", indent);

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
