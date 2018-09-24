pub fn build_subcommand<'a>(subcommand: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
	clap_app!(@app (subcommand)
		(about: "Remove mods.")
		(@arg mods: ... +required index(1) "mod names to remove"))
}

pub async fn run<'a>(
	matches: &'a clap::ArgMatches<'a>,
	local_api: crate::Result<&'a factorio_mods_local::API>,
	web_api: crate::Result<&'a factorio_mods_web::API>,
	config_file_path: Option<std::path::PathBuf>,
	prompt_override: Option<bool>,
) -> crate::Result<()> {
	let mods = matches.values_of("mods").unwrap();

	let local_api = local_api?;
	let web_api = web_api?;

	let mut config = crate::config::Config::load(local_api, config_file_path)?;

	for mod_ in mods {
		let name = factorio_mods_common::ModName(mod_.to_string());
		config.mods.remove(&name);
	}

	await!(crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

	Ok(())
}
