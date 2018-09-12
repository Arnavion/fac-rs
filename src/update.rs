pub fn build_subcommand<'a>(subcommand: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
	clap_app!(@app (subcommand)
		(about: "Update installed mods."))
}

pub async fn run<'a>(
	local_api: crate::Result<&'a factorio_mods_local::API>,
	web_api: crate::Result<&'a factorio_mods_web::API>,
	prompt_override: Option<bool>,
) -> crate::Result<()> {
	let local_api = local_api?;
	let web_api = web_api?;

	let config = crate::config::Config::load(local_api)?;

	await!(crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

	Ok(())
}
