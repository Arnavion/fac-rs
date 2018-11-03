#[derive(Debug, structopt_derive::StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::VersionlessSubcommands"))]
pub enum SubCommand {
	#[structopt(name = "cleanup", about = "Delete the local mods cache")]
	Cleanup,

	#[structopt(name = "list", about = "List mods in the local cache")]
	List,
}

impl SubCommand {
	pub async fn run<'a>(
		self,
		local_api: crate::Result<&'a factorio_mods_local::API>,
		config_file_path: Option<std::path::PathBuf>,
	) -> crate::Result<()> {
		use crate::ResultExt;

		let local_api = local_api?;

		let config = crate::config::Config::load(local_api, config_file_path)?;

		let cache_directory = config.cache_directory()?;

		match self {
			SubCommand::Cleanup => match std::fs::remove_dir_all(cache_directory) {
				Ok(()) => Ok(()),
				Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
				Err(err) => Err(err).chain_err(|| "Could not delete cache directory")?,
			},

			SubCommand::List => {
				let directory_entries = match std::fs::read_dir(cache_directory) {
					Ok(directory_entries) => directory_entries,
					Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
					Err(err) => return Err(err).chain_err(|| "Could not read cache directory")?,
				};
				for directory_entry in directory_entries {
					let directory_entry = directory_entry.chain_err(|| "Could not read cache directory")?;
					let path = directory_entry.path();
					let mod_ = factorio_mods_local::InstalledMod::parse(path).chain_err(|| "Could not parse cached mod")?;

					println!("    {} {}", mod_.info.name, mod_.info.version);
				}

				Ok(())
			},
		}
	}
}
