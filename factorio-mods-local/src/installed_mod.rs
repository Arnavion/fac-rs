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
			ensure!(path.extension() == Some("zip".as_ref()), ::ErrorKind::UnknownModFormat(path));

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
) -> ::Result<impl Iterator<Item = ::Result<InstalledMod>> + 'static> {
	let directory_entries = ::std::fs::read_dir(mods_directory)?;

	let name_pattern = name_pattern.unwrap_or("*");
	let matcher = ::globset::Glob::new(name_pattern).map_err(|err| ::ErrorKind::Pattern(name_pattern.to_string(), err))?.compile_matcher();

	Ok(GenIterator(move || for directory_entry in directory_entries {
		match directory_entry {
			Ok(directory_entry) => {
				let path = directory_entry.path();

				let matches = path.file_name().map_or(false, |filename| matcher.is_match(filename));
				if !matches {
					continue;
				}

				// TODO: Workaround for https://github.com/rust-lang/rust/issues/44197
				let installed_mod = InstalledMod::parse(path, &mod_status);
				let installed_mod = match installed_mod {
					Ok(installed_mod) => installed_mod,

					Err(::Error(::ErrorKind::UnknownModFormat(_), _)) => continue,

					Err(err) => {
						yield Err(err);
						continue;
					},
				};

				if let Some(ref version) = version {
					if version != installed_mod.info().version() {
						continue;
					}
				}

				yield Ok(installed_mod);
			},

			Err(err) =>
				yield Err(err.into()),
		}
	}))
}

struct GenIterator<G>(G);

impl<G> Iterator for GenIterator<G> where G: ::std::ops::Generator<Return = ()> {
	type Item = G::Yield;

	fn next(&mut self) -> Option<Self::Item> {
		match unsafe { ::std::ops::Generator::resume(&mut self.0) } {
			::std::ops::GeneratorState::Yielded(value) => Some(value),
			::std::ops::GeneratorState::Complete(()) => None,
		}
	}
}
