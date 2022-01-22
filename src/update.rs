#[derive(clap::Parser)]
pub(crate) struct SubCommand {
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::Api,
		web_api: &factorio_mods_web::Api,
		config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> anyhow::Result<()> {
		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
