#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
}

impl SubCommand {
	pub(crate) async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, crate::Error>,
		web_api: Result<&'a factorio_mods_web::API, crate::Error>,
		config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		let local_api = local_api?;
		let web_api = web_api?;

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
