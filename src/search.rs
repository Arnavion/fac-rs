pub fn build_subcommand<'a>(subcommand: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
	clap_app!(@app (subcommand)
		(about: "Search the mods database.")
		(@arg query: index(1) "search string"))
}

pub async fn run<'a>(
	matches: &'a clap::ArgMatches<'a>,
	web_api: crate::Result<&'a factorio_mods_web::API>,
) -> crate::Result<()> {
	use crate::ResultExt;

	let query = matches.value_of("query").unwrap_or("");

	let web_api = web_api?;

	let mut mods = web_api.search(query);
	let mut mods = std::pin::PinMut::new(&mut mods);

	while let Some(mod_) = await!(futures::StreamExt::next(&mut *mods)) {
		let mod_ = mod_.chain_err(|| "Could not retrieve mods")?;

		println!("{}", mod_.title);
		println!("    Name: {}", mod_.name);
		println!();
		crate::util::wrapping_println(&mod_.summary.0, "    ");
		println!();
	}

	Ok(())
}
