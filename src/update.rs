#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
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

		let config = crate::config::Config::load(local_api, config_file_path)?;

		await!(crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

		Ok(())
	}
}
