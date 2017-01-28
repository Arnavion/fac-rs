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
		let (info, mod_type): (::factorio_mods_common::ModInfo, _) = if path.is_file() {
			if match path.extension() {
				Some(extension) if extension == "zip" => true,
				_ => false,
			} {
				let zip_file = match ::std::fs::File::open(&path) {
					Ok(zip_file) => zip_file,
					Err(err) => bail!(::ErrorKind::IO(path, err)),
				};

				let mut zip_file = match ::zip::ZipArchive::new(zip_file) {
					Ok(zip_file) => zip_file,
					Err(err) => bail!(::ErrorKind::Zip(path, err)),
				};

				if zip_file.len() == 0 {
					bail!(::ErrorKind::EmptyZippedMod(path));
				}

				let toplevel = {
					let first_file = match zip_file.by_index(0) {
						Ok(first_file) => first_file,
						Err(err) => bail!(::ErrorKind::Zip(path, err)),
					};

					first_file.name().split('/').next().unwrap().to_string()
				};

				let info_json_file_path = format!("{}/info.json", toplevel);

				let info_json_file = match zip_file.by_name(&info_json_file_path) {
					Ok(info_json_file) => info_json_file,
					Err(err) => bail!(::ErrorKind::Zip(path, err)),
				};

				match ::serde_json::from_reader(info_json_file) {
					Ok(info) => (info, InstalledModType::Zipped),
					Err(err) => bail!(::ErrorKind::JSON(path, err)),
				}
			}
			else {
				bail!(::ErrorKind::UnknownModFormat(path));
			}
		}
		else {
			let info_json_file_path = path.join("info.json");

			let info_json_file = match ::std::fs::File::open(&info_json_file_path) {
				Ok(info_json_file) => info_json_file,
				Err(err) => match err.kind() {
					::std::io::ErrorKind::NotFound => bail!(::ErrorKind::UnknownModFormat(info_json_file_path)),
					_ => bail!(::ErrorKind::IO(info_json_file_path, err)),
				},
			};

			match ::serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Unpacked),
				Err(err) => bail!(::ErrorKind::JSON(info_json_file_path, err)),
			}
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
