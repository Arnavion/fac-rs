/// An installed mod object.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters)]
pub struct InstalledMod {
	/// The path of the mod.
	path: ::std::path::PathBuf,

	/// The info.json of the mod
	info: ::factorio_mods_common::ModInfo,

	/// Whether the installed mod is enabled or not in `mod-list.json`
	enabled: bool,

	/// Whether the installed mod is zipped or unpacked.
	#[getter(copy)]
	mod_type: InstalledModType,
}

/// The type of an installed mod.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
	) -> ::Result<Self> {
		let (info, mod_type): (::factorio_mods_common::ModInfo, _) = if path.is_file() {
			if match path.extension() {
				Some(extension) if extension == "zip" => true,
				_ => false,
			} {
				let zip_file = match ::std::fs::File::open(&path) {
					Ok(zip_file) => zip_file,
					Err(err) => bail!(::ErrorKind::FileIO(path, err)),
				};

				let mut zip_file = match ::zip::ZipArchive::new(zip_file) {
					Ok(zip_file) => zip_file,
					Err(err) => bail!(::ErrorKind::Zip(path, err)),
				};

				ensure!(zip_file.len() != 0, ::ErrorKind::EmptyZippedMod(path));

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
					Err(err) => bail!(::ErrorKind::ReadJSONFile(path, err)),
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
					_ => bail!(::ErrorKind::FileIO(info_json_file_path, err)),
				},
			};

			match ::serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Unpacked),
				Err(err) => bail!(::ErrorKind::ReadJSONFile(info_json_file_path, err)),
			}
		};

		let enabled = mod_status.get(info.name());

		Ok(InstalledMod::new(path, info, enabled.cloned().unwrap_or(true), mod_type))
	}
}

/// Constructs an iterator over all the locally installed mods.
pub fn find(
	mods_directory: &::std::path::Path,
	name_pattern: Option<&str>,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
) -> ::Result<impl Iterator<Item = ::Result<InstalledMod>>> {
	let directory_entries = ::std::fs::read_dir(mods_directory)?;

	let name_pattern = name_pattern.unwrap_or("*");
	let matcher = ::globset::Glob::new(name_pattern).map_err(|err| ::ErrorKind::Pattern(name_pattern.to_string(), err))?.compile_matcher();

	Ok(InstalledModIterator {
		directory_entries,
		matcher,
		version,
		mod_status,
		ended: false,
	})
}

/// An iterator over all the locally installed mods.
struct InstalledModIterator {
	directory_entries: ::std::fs::ReadDir,
	matcher: ::globset::GlobMatcher,
	version: Option<::factorio_mods_common::ReleaseVersion>,
	mod_status: ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
	ended: bool,
}

impl Iterator for InstalledModIterator {
	type Item = ::Result<InstalledMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.ended {
			return None;
		}

		loop {
			match self.directory_entries.next() {
				Some(Ok(directory_entry)) => {
					let path = directory_entry.path();

					let matches =
						if let Some(filename) = path.file_name() {
							self.matcher.is_match(filename)
						}
						else {
							false
						};

					if !matches {
						continue;
					}

					let installed_mod = match InstalledMod::parse(path, &self.mod_status) {
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

				Some(Err(err)) =>
					return Some(Err(::ErrorKind::IO(err).into())),

				None =>
					return None,
			}
		}
	}
}
