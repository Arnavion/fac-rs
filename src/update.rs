#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		local_api: Result<&'a factorio_mods_local::API, failure::Error>,
		web_api: Result<&'a factorio_mods_web::API, failure::Error>,
		config_file_path: Option<std::path::PathBuf>,
		prompt_override: Option<bool>,
	) -> Result<(), failure::Error> {
		let local_api = local_api?;
		let web_api = web_api?;

		let config = crate::config::Config::load(local_api, config_file_path)?;

		await!(crate::solve::compute_and_apply_diff(local_api, web_api, config, prompt_override))?;

		Ok(())
	}
}
