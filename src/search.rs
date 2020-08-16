#[derive(Debug, structopt::StructOpt)]
pub(crate) struct SubCommand {
	#[structopt(help = "search string", default_value = "")]
	query: String,
}

impl SubCommand {
	pub(crate) async fn run(
		self,
		web_api: &factorio_mods_web::API,
	) -> Result<(), crate::Error> {
		use crate::ResultExt;

		let mut mods = web_api.search(&self.query);

		while let Some(mod_) = futures_util::StreamExt::next(&mut mods).await {
			let mod_ = mod_.context("could not retrieve mods")?;

			println!("{}", mod_.title);
			println!("    Name: {}", mod_.name);
			println!();
			crate::util::wrapping_println(&mod_.summary.0, "    ");
			println!();
		}

		Ok(())
	}
}
