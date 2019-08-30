#[derive(Debug, structopt_derive::StructOpt)]
pub(crate) struct SubCommand {
}

impl SubCommand {
	pub(crate) async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, failure::Error>,
		web_api: Result<&'a factorio_mods_web::API, failure::Error>,
		config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> Result<(), failure::Error> {
		let local_api = local_api?;
		let web_api = web_api?;

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
