#[derive(Debug)]
pub enum InstalledMod {
	Zipped {
		name: ::factorio_mods_common::ModName,
		version: ::factorio_mods_common::ReleaseVersion,
		game_version: ::factorio_mods_common::GameVersion,
		enabled: bool,
	},

	Unpacked {
		name: ::factorio_mods_common::ModName,
		version: ::factorio_mods_common::ReleaseVersion,
		game_version: ::factorio_mods_common::GameVersion,
		enabled: bool,
	},
}

impl InstalledMod {
	pub fn find<'a>(
		mods_directory: &::std::path::Path,
		name_pattern: Option<&str>,
		version: Option<::factorio_mods_common::ReleaseVersion>,
		mod_status: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	) -> ::Result<InstalledModIterator<'a>> {
		let glob_pattern = mods_directory.join("*");

		let paths =
			glob_pattern.to_str()
				.map(::glob::glob)
				.ok_or_else(|| ::ErrorKind::Utf8Path(glob_pattern))??;

		let name_pattern = if let Some(name_pattern) = name_pattern {
			Some(::glob::Pattern::new(name_pattern)?)
		}
		else {
			None
		};

		Ok(InstalledModIterator {
			paths: paths,
			name_pattern: name_pattern,
			version: version,
			mod_status: mod_status,
			errored: false,
		})
	}

	pub fn new(
		path: ::std::path::PathBuf,
		mod_status: &::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	) -> ::Result<InstalledMod> {
		let info: ModInfo =
			if path.is_file() {
				match path.extension() {
					Some(extension) if extension == "zip" => {
						let zip_file = ::std::fs::File::open(&path)?;
						let mut zip_file = ::zip::ZipArchive::new(zip_file)?;
						if zip_file.len() == 0 {
							return Err(::ErrorKind::EmptyZippedMod(path.clone()).into());
						}

						let toplevel = {
							let first_file = zip_file.by_index(0)?;
							first_file.name().split('/').next().unwrap().to_string()
						};
						let info_json_file_path = format!("{}/info.json", toplevel);
						let info_json_file = zip_file.by_name(&info_json_file_path)?;
						::serde_json::from_reader(info_json_file)?
					},

					_ => return Err(::ErrorKind::UnknownModFormat.into()),
				}
			}
			else {
				let info_json_file_path = path.join("info.json");
				let info_json_file =
					::std::fs::File::open(&info_json_file_path).map_err(|err| {
						match err.kind() {
							::std::io::ErrorKind::NotFound => ::ErrorKind::UnknownModFormat.into(),
							_ => ::Error::from(err),
						}
					})?;
				::serde_json::from_reader(info_json_file)?
			};

		let enabled = mod_status.get(&info.name);

		Ok(InstalledMod::Zipped {
			name: info.name,
			version: info.version,
			game_version: info.factorio_version,
			enabled: enabled.cloned().unwrap_or(true),
		})
	}

	pub fn name(&self) -> &::factorio_mods_common::ModName {
		match *self {
			InstalledMod::Zipped { ref name, .. } |
			InstalledMod::Unpacked { ref name, .. } => name,
		}
	}

	pub fn version(&self) -> &::factorio_mods_common::ReleaseVersion {
		match *self {
			InstalledMod::Zipped { ref version, .. } |
			InstalledMod::Unpacked { ref version, .. }=> version,
		}
	}

	pub fn game_version(&self) -> &::factorio_mods_common::GameVersion {
		match *self {
			InstalledMod::Zipped { ref game_version, .. } |
			InstalledMod::Unpacked { ref game_version, .. } => game_version,
		}
	}

	pub fn enabled(&self) -> &bool {
		match *self {
			InstalledMod::Zipped { ref enabled, .. } |
			InstalledMod::Unpacked { ref enabled, .. } => enabled,
		}
	}
}

pub struct InstalledModIterator<'a> {
	paths: ::glob::Paths,
	name_pattern: Option<::glob::Pattern>,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	errored: bool,
}

impl<'a> Iterator for InstalledModIterator<'a> {
	type Item = ::Result<InstalledMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.errored {
			return None;
		}

		loop {
			match self.paths.next() {
				Some(Ok(path)) => {
					let installed_mod = match InstalledMod::new(path, self.mod_status) {
						Ok(installed_mod) => installed_mod,

						Err(err) => match *err.kind() {
							::ErrorKind::UnknownModFormat => continue,
							_ => {
								self.errored = true;
								return Some(Err(err));
							}
						},
					};

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
					return Some(Err(::ErrorKind::Glob(err).into()));
				},

				None => {
					return None;
				},
			}
		}
	}
}

lazy_static! {
	static ref DEFAULT_GAME_VERSION: ::factorio_mods_common::GameVersion = ::factorio_mods_common::GameVersion::new(::semver::VersionReq::parse("0.12").unwrap());
}

fn default_game_version() -> ::factorio_mods_common::GameVersion {
	DEFAULT_GAME_VERSION.clone()
}

make_struct!(ModInfo {
	name: ::factorio_mods_common::ModName,
	author: ::factorio_mods_common::AuthorNames,
	title: ::factorio_mods_common::ModTitle,
	description: ::factorio_mods_common::ModDescription,

	version: ::factorio_mods_common::ReleaseVersion,
	#[serde(default = "default_game_version")]
	factorio_version: ::factorio_mods_common::GameVersion,

	homepage: Option<::factorio_mods_common::Url>,
});
