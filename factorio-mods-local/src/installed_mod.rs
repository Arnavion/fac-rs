/// An installed mod object.
#[derive(Clone, Debug, new, getters)]
pub struct InstalledMod {
	/// The path of the mod.
	path: ::std::path::PathBuf,

	/// The info.json of the mod
	info: ::factorio_mods_common::ModInfo,

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
		path: ::std::path::PathBuf,
		mod_status: &::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	) -> ::Result<InstalledMod> {
		let (info, mod_type): (::factorio_mods_common::ModInfo, _) =
			if path.is_file() {
				match path.extension() {
					Some(extension) if extension == "zip" => {
						let zip_file = ::std::fs::File::open(&path).map_err(|err| ::ErrorKind::IO(path.clone(), err))?;
						let mut zip_file = ::zip::ZipArchive::new(zip_file).map_err(|err| ::ErrorKind::Zip(path.clone(), err))?;
						if zip_file.len() == 0 {
							bail!(::ErrorKind::EmptyZippedMod(path.clone()));
						}

						let toplevel = {
							let first_file = zip_file.by_index(0).map_err(|err| ::ErrorKind::Zip(path.clone(), err))?;
							first_file.name().split('/').next().unwrap().to_string()
						};
						let info_json_file_path = format!("{}/info.json", toplevel);
						let info_json_file = zip_file.by_name(&info_json_file_path).map_err(|err| ::ErrorKind::Zip(path.clone(), err))?;
						(::serde_json::from_reader(info_json_file).map_err(|err| ::ErrorKind::JSON(path.clone(), err))?, InstalledModType::Zipped)
					},

					_ => bail!(::ErrorKind::UnknownModFormat(path.clone())),
				}
			}
			else {
				let info_json_file_path = path.join("info.json");
				let info_json_file =
					::std::fs::File::open(&info_json_file_path).map_err(|err| {
						let info_json_file_path = info_json_file_path.clone();

						match err.kind() {
							::std::io::ErrorKind::NotFound => ::ErrorKind::UnknownModFormat(info_json_file_path),
							_ => ::ErrorKind::IO(info_json_file_path, err),
						}
					})?;
				(::serde_json::from_reader(info_json_file).map_err(|err| ::ErrorKind::JSON(info_json_file_path.clone(), err))?, InstalledModType::Unpacked)
			};

		let enabled = mod_status.get(info.name());

		Ok(InstalledMod::new(path, info, enabled.cloned().unwrap_or(true), mod_type))
	}
}

/// Constructs an iterator over all the locally installed mods.
pub fn find<'a>(
	mods_directory: &::std::path::Path,
	name_pattern: Option<&str>,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
) -> ::Result<impl Iterator<Item = ::Result<InstalledMod>> + 'a> {
	let name_pattern = name_pattern.unwrap_or("*");
	let glob_pattern = mods_directory.join(name_pattern).into_os_string().into_string().map_err(::ErrorKind::Utf8Path)?;
	let paths = ::glob::glob(&glob_pattern).map_err(|err| ::ErrorKind::Pattern(glob_pattern, err))?;

	Ok(InstalledModIterator {
		paths: paths,
		version: version,
		mod_status: mod_status,
		ended: false,
	})
}

/// An iterator over all the locally installed mods.
struct InstalledModIterator<'a> {
	paths: ::glob::Paths,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: &'a ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	ended: bool,
}

impl<'a> Iterator for InstalledModIterator<'a> {
	type Item = ::Result<InstalledMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.ended {
			return None;
		}

		loop {
			match self.paths.next() {
				Some(Ok(path)) => {
					let installed_mod = match InstalledMod::parse(path, self.mod_status) {
						Ok(installed_mod) => installed_mod,

						Err(::Error(::ErrorKind::UnknownModFormat(_), _)) => continue,

						Err(err) => {
							self.ended = true;
							return Some(Err(err));
						},
					};

					if let Some(ref version) = self.version {
						if version != installed_mod.info().version() {
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
