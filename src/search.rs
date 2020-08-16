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

		while let Some(mod_) = futures_util::TryStreamExt::try_next(&mut mods).await.context("could not retrieve mods")? {
			println!("{}", mod_.title);
			println!("    Name: {}", mod_.name);
			println!();
			crate::util::wrapping_println(&mod_.summary.0, "    ");
			println!();
		}

		Ok(())
	}
}
