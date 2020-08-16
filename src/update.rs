#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::API,
		web_api: &factorio_mods_web::API,
		config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
