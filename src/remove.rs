#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
	#[structopt(help = "mod names to remove", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl SubCommand {
	pub(crate) async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, crate::Error>,
		web_api: Result<&'a factorio_mods_web::API, crate::Error>,
		mut config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> Result<(), crate::Error> {
		let local_api = local_api?;
		let web_api = web_api?;

		let mods = config.mods.as_mut().unwrap();

		for mod_ in self.names {
			mods.remove(&mod_);
		}

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
