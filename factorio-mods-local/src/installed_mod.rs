use types;

#[derive(Debug)]
pub enum InstalledMod {
	Zipped {
		name: ::factorio_mods_common::ModName,
		version: ::factorio_mods_common::ReleaseVersion,
		game_version: ::factorio_mods_common::GameVersion,
	},

	Unpacked {
		name: ::factorio_mods_common::ModName,
		version: ::factorio_mods_common::ReleaseVersion,
		game_version: ::factorio_mods_common::GameVersion,
	},
}

impl InstalledMod {
	pub fn find<'a>(
		mods_directory: &'a ::std::path::Path,
		name_pattern: Option<&str>,
		version: Option<::factorio_mods_common::ReleaseVersion>,
	) -> Result<InstalledModIterator, types::LocalError> {
		let glob_pattern = mods_directory.join("*.zip");

		let paths = try!(try!(
			glob_pattern.to_str()
				.map(|v| ::glob::glob(v).map_err(types::LocalError::pattern))
				.ok_or_else(|| types::LocalError::utf8_path(glob_pattern))));

		let name_pattern = if let Some(name_pattern) = name_pattern {
			Some(try!(::glob::Pattern::new(&name_pattern).map_err(types::LocalError::pattern)))
		}
		else {
			None
		};

		Ok(InstalledModIterator {
			paths: paths,
			name_pattern: name_pattern,
			version: version,
		})
	}

	pub fn new(path: ::std::path::PathBuf) -> InstalledMod {
		if path.is_file() {
			if let Some(extension) = path.extension() {
				if extension == "zip" {
					if let Ok(zip_file) = ::std::fs::File::open(&path) {
						if let Ok(mut zip_file) = ::zip::ZipArchive::new(zip_file) {
							if zip_file.len() > 0 {
								let toplevel = if let Ok(first_file) = zip_file.by_index(0) {
									Some(first_file.name().split('/').next().unwrap().to_string())
								}
								else {
									None
								};

								if let Some(toplevel) = toplevel {
									let info_json_file_path = format!("{}/info.json", toplevel);
									if let Ok(info_json_file) = zip_file.by_name(&info_json_file_path) {
										if let Ok(info) = ::serde_json::from_reader::<::zip::read::ZipFile, ModInfo>(info_json_file) {
											return InstalledMod::Zipped {
												name: info.name,
												version: info.version,
												game_version: info.factorio_version.unwrap_or_else(|| ::factorio_mods_common::GameVersion("0.12".to_string())),
											};
										}
									}
								}
							}
						}
					}
				}
			}
		}

		unimplemented!();
	}

	pub fn name(&self) -> &::factorio_mods_common::ModName {
		match self {
			&InstalledMod::Zipped { ref name, .. } => name,
			&InstalledMod::Unpacked { ref name, .. } => name,
		}
	}

	pub fn version(&self) -> &::factorio_mods_common::ReleaseVersion {
		match self {
			&InstalledMod::Zipped { ref version, .. } => version,
			&InstalledMod::Unpacked { ref version, .. } => version,
		}
	}

	pub fn game_version(&self) -> &::factorio_mods_common::GameVersion {
		match self {
			&InstalledMod::Zipped { ref game_version, .. } => game_version,
			&InstalledMod::Unpacked { ref game_version, .. } => game_version,
		}
	}
}

pub struct InstalledModIterator {
	paths: ::glob::Paths,
	name_pattern: Option<::glob::Pattern>,
	version: Option<::factorio_mods_common::ReleaseVersion>,
}

impl Iterator for InstalledModIterator {
	type Item = Result<InstalledMod, types::LocalError>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.paths.next() {
				Some(Ok(path)) => {
					let installed_mod = InstalledMod::new(path);

					if let Some(ref name_pattern) = self.name_pattern {
						if !name_pattern.matches(installed_mod.name()) {
							continue;
						}
					}

					if let Some(ref version) = self.version {
						if version != installed_mod.version() {
							continue;
						}
					}

					return Some(Ok(installed_mod));
				},

				Some(Err(err)) => {
					return Some(Err(types::LocalError::glob(err)));
				},

				None => {
					return None;
				},
			}
		}
	}
}

make_deserializable!(struct ModInfo {
	name: ::factorio_mods_common::ModName,
	author: ::factorio_mods_common::AuthorNames,
	title: ::factorio_mods_common::ModTitle,
	description: ::factorio_mods_common::ModDescription,

	version: ::factorio_mods_common::ReleaseVersion,
	factorio_version: Option<::factorio_mods_common::GameVersion>,

	homepage: Option<::factorio_mods_common::Url>,
});
