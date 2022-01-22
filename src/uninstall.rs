#[derive(clap::Parser)]
pub(crate) struct SubCommand {
	#[clap(help = "mods to uninstall", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		local_api: &factorio_mods_local::Api,
		web_api: &factorio_mods_web::Api,
		mut config: crate::config::Config,
		prompt_override: Option<bool>,
	) -> anyhow::Result<()> {
		let mods = config.mods.as_mut().unwrap();

		for mod_ in self.names {
			mods.remove(&mod_);
		}

		crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override).await?;

		Ok(())
	}
}
