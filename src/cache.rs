pub fn build_subcommand<'a>(subcommand: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
	clap_app!(@app (subcommand)
		(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands)
		(about: "Manage the local mods cache.")
		(@subcommand cleanup =>
			(about: "Delete the local mods cache."))
		(@subcommand list =>
			(about: "List mods in the local cache.")))
}

pub async fn run<'a>(
	matches: &'a clap::ArgMatches<'a>,
	local_api: crate::Result<&'a factorio_mods_local::API>,
	config_file_path: Option<std::path::PathBuf>,
) -> crate::Result<()> {
	use crate::ResultExt;

	let subcommand_name = matches.subcommand_name();

	let local_api = local_api?;

	let config = crate::config::Config::load(local_api, config_file_path)?;

	let cache_directory = config.cache_directory()?;

	match subcommand_name {
		Some("cleanup") => match std::fs::remove_dir_all(cache_directory) {
			Ok(()) => Ok(()),
			Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
			Err(err) => Err(err).chain_err(|| "Could not delete cache directory")?,
		},

		Some("list") => {
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

		_ => unreachable!(),
	}
}
