#[derive(Debug, structopt_derive::StructOpt)]
pub struct SubCommand {
	#[structopt(help = "search string", default_value = "")]
	query: String,
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		web_api: crate::Result<&'a factorio_mods_web::API>,
	) -> crate::Result<()> {
		use crate::ResultExt;

		let web_api = web_api?;

		let mut mods = web_api.search(&self.query);
		let mut mods = std::pin::Pin::new(&mut mods);

		while let Some(mod_) = await!(futures::StreamExt::next(&mut *mods)) {
			let mod_ = mod_.chain_err(|| "Could not retrieve mods")?;

			println!("{}", mod_.title);
			println!("    Name: {}", mod_.name);
			println!();
			crate::util::wrapping_println(&mod_.summary.0, "    ");
			println!();
		}

		Ok(())
	}
}
