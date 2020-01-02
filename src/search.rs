#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
	#[structopt(help = "search string", default_value = "")]
	query: String,
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		web_api: Result<&'_ factorio_mods_web::API, failure::Error>,
	) -> Result<(), failure::Error> {
		use failure::ResultExt;

		let web_api = web_api?;

		let mut mods = web_api.search(&self.query);

		while let Some(mod_) = futures_util::StreamExt::next(&mut mods).await {
			let mod_ = mod_.context("Could not retrieve mods")?;

			println!("{}", mod_.title);
			println!("    Name: {}", mod_.name);
			println!();
			crate::util::wrapping_println(&mod_.summary.0, "    ");
			println!();
		}

		Ok(())
	}
}
