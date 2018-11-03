#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
	#[structopt(help = "mod names to remove", required = true)]
	names: Vec<factorio_mods_common::ModName>,
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		local_api: crate::Result<&'a factorio_mods_local::API>,
		web_api: crate::Result<&'a factorio_mods_web::API>,
		config_file_path: Option<std::path::PathBuf>,
		prompt_override: Option<bool>,
	) -> crate::Result<()> {
		let local_api = local_api?;
		let web_api = web_api?;

		let mut config = crate::config::Config::load(local_api, config_file_path)?;

		for mod_ in self.names {
			let name = factorio_mods_common::ModName(mod_.to_string());
			config.mods.remove(&name);
		}

		await!(crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

		Ok(())
	}
}
