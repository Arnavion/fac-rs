/// An installed mod object.
#[derive(Clone, Debug, new, getters)]
pub struct InstalledMod {
	/// The name of the installed mod.
	name: ::factorio_mods_common::ModName,

	/// The version of the installed mod.
	version: ::factorio_mods_common::ReleaseVersion,

	/// The game version of the installed mod.
	game_version: ::factorio_mods_common::GameVersion,

	/// Whether the installed mod is enabled or not in `mod-list.json`
	enabled: bool,

	/// Whether the installed mod is zipped or unpacked.
	mod_type: InstalledModType,
}

/// The type of an installed mod.
#[derive(Clone, Debug)]
pub enum InstalledModType {
	/// A zipped mod.
	Zipped,

	/// An unpacked mod.
	Unpacked,
}

impl InstalledMod {
	/// Parses the installed mod at the given location.
	pub fn parse(
		path: &::std::path::Path,
		mod_status: &::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	) -> ::Result<InstalledMod> {
		let (info, mod_type): (::factorio_mods_common::ModInfo, _) =
			if path.is_file() {
				match path.extension() {
					Some(extension) if extension == "zip" => {
						let zip_file = ::std::fs::File::open(path)?;
						let mut zip_file = ::zip::ZipArchive::new(zip_file)?;
						if zip_file.len() == 0 {
							return Err(::ErrorKind::EmptyZippedMod(path.into()).into());
						}

						let toplevel = {
							let first_file = zip_file.by_index(0)?;
							first_file.name().split('/').next().unwrap().to_string()
						};
						let info_json_file_path = format!("{}/info.json", toplevel);
						let info_json_file = zip_file.by_name(&info_json_file_path)?;
						(::serde_json::from_reader(info_json_file)?, InstalledModType::Zipped)
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
				(::serde_json::from_reader(info_json_file)?, InstalledModType::Unpacked)
			};

		let enabled = mod_status.get(info.name());

		Ok(InstalledMod::new(
			info.name().clone(),
			info.version().clone(),
			info.factorio_version().clone(),
			enabled.cloned().unwrap_or(true),
			mod_type))
	}
}

/// Constructs an iterator over all the locally installed mods.
pub fn find<'a>(
	mods_directory: &::std::path::Path,
	name_pattern: Option<&str>,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
) -> ::Result<impl Iterator<Item = ::Result<InstalledMod>> + 'a> {
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

/// An iterator over all the locally installed mods.
struct InstalledModIterator<'a> {
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
					let installed_mod = match InstalledMod::parse(&path, self.mod_status) {
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
