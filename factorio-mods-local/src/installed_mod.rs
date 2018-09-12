/// An installed mod object.
#[derive(Clone, Debug, PartialEq)]
pub struct InstalledMod {
	/// The path of the mod.
	pub path: std::path::PathBuf,

	/// The info.json of the mod
	pub info: ModInfo,

	/// Whether the installed mod is zipped or unpacked.
	pub mod_type: InstalledModType,
}

/// Represents the contents of `info.json` of a mod release.
#[derive(Clone, Debug, PartialEq, serde_derive::Deserialize)]
pub struct ModInfo {
	/// The name of the mod release.
	pub name: factorio_mods_common::ModName,

	/// The authors of the mod release.
	#[serde(deserialize_with = "factorio_mods_common::deserialize_string_or_seq_string")]
	pub author: Vec<factorio_mods_common::AuthorName>,

	/// The title of the mod release.
	pub title: factorio_mods_common::ModTitle,

	/// A longer description of the mod release.
	pub description: Option<factorio_mods_common::ModDescription>,

	/// The version of the mod release.
	pub version: factorio_mods_common::ReleaseVersion,

	/// The versions of the game supported by the mod release.
	#[serde(default = "default_game_version")]
	pub factorio_version: factorio_mods_common::ModVersionReq,

	/// The URL of the homepage of the mod release.
	pub homepage: Option<factorio_mods_common::Url>,

	/// Dependencies
	#[serde(default = "default_dependencies")]
	#[serde(deserialize_with = "factorio_mods_common::deserialize_string_or_seq_string")]
	pub dependencies: Vec<factorio_mods_common::Dependency>,
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
	pub fn parse(path: std::path::PathBuf) -> crate::Result<Self> {
		let (info, mod_type): (ModInfo, _) = if path.is_file() {
			if path.extension() != Some("zip".as_ref()) {
				error_chain::bail!(crate::ErrorKind::UnknownModFormat(path));
			}

			let zip_file = match std::fs::File::open(&path) {
				Ok(zip_file) => zip_file,
				Err(err) => error_chain::bail!(crate::ErrorKind::FileIO(path, err)),
			};

			let mut zip_file = match zip::ZipArchive::new(zip_file) {
				Ok(zip_file) => zip_file,
				Err(err) => error_chain::bail!(crate::ErrorKind::Zip(path, err)),
			};

			if zip_file.len() == 0 {
				error_chain::bail!(crate::ErrorKind::EmptyZippedMod(path));
			}

			let toplevel = {
				let first_file = match zip_file.by_index(0) {
					Ok(first_file) => first_file,
					Err(err) => error_chain::bail!(crate::ErrorKind::Zip(path, err)),
				};

				first_file.name().split('/').next().unwrap().to_string()
			};

			let info_json_file_path = format!("{}/info.json", toplevel);

			let info_json_file = match zip_file.by_name(&info_json_file_path) {
				Ok(info_json_file) => info_json_file,
				Err(err) => error_chain::bail!(crate::ErrorKind::Zip(path, err)),
			};

			match serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Zipped),
				Err(err) => error_chain::bail!(crate::ErrorKind::ReadJSONFile(path, err)),
			}
		}
		else {
			let info_json_file_path = path.join("info.json");

			let info_json_file = match std::fs::File::open(&info_json_file_path) {
				Ok(info_json_file) => info_json_file,
				Err(err) => match err.kind() {
					std::io::ErrorKind::NotFound => error_chain::bail!(crate::ErrorKind::UnknownModFormat(info_json_file_path)),
					_ => error_chain::bail!(crate::ErrorKind::FileIO(info_json_file_path, err)),
				},
			};

			match serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Unpacked),
				Err(err) => error_chain::bail!(crate::ErrorKind::ReadJSONFile(info_json_file_path, err)),
			}
		};

		Ok(InstalledMod { path, info, mod_type })
	}
}

/// Constructs an iterator over all the locally installed mods.
pub fn find(
	mods_directory: &std::path::Path,
	name_pattern: Option<String>,
	version: Option<factorio_mods_common::ReleaseVersion>,
) -> crate::Result<impl Iterator<Item = crate::Result<InstalledMod>> + 'static> {
	let directory_entries = std::fs::read_dir(mods_directory)?;

	let name_pattern = name_pattern.map_or(std::borrow::Cow::Borrowed("*"), std::borrow::Cow::Owned);
	let matcher = globset::Glob::new(&name_pattern).map_err(|err| crate::ErrorKind::Pattern(name_pattern.into_owned(), err))?.compile_matcher();

	Ok(GenIterator(move || for directory_entry in directory_entries {
		match directory_entry {
			Ok(directory_entry) => {
				let path = directory_entry.path();

				let matches = path.file_name().map_or(false, |filename| matcher.is_match(filename));
				if !matches {
					continue;
				}

				let installed_mod = match InstalledMod::parse(path) {
					Ok(installed_mod) => installed_mod,

					Err(crate::Error(crate::ErrorKind::UnknownModFormat(_), _)) => continue,

					Err(err) => {
						yield Err(err);
						continue;
					},
				};

				if let Some(ref version) = version {
					if version != &installed_mod.info.version {
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

lazy_static! {
	static ref DEFAULT_GAME_VERSION: factorio_mods_common::ModVersionReq = factorio_mods_common::ModVersionReq("0.12".parse().unwrap());
	static ref DEFAULT_DEPENDENCIES: Vec<factorio_mods_common::Dependency> = vec![factorio_mods_common::Dependency {
		name: factorio_mods_common::ModName("base".to_string()),
		version: factorio_mods_common::ModVersionReq(semver::VersionReq::any()),
		required: true,
	}];
}

/// Generates a copy of the default game version.
///
/// Used as the default value of the `factorio_version` field in a mod's `info.json` if the field doesn't exist.
fn default_game_version() -> factorio_mods_common::ModVersionReq {
	DEFAULT_GAME_VERSION.clone()
}

/// The default dependencies of a mod.
///
/// Used as the default value of the `dependencies` field in a mod's `info.json` if the field doesn't exist.
fn default_dependencies() -> Vec<factorio_mods_common::Dependency> {
	DEFAULT_DEPENDENCIES.clone()
}

struct GenIterator<G>(G);

impl<G> Iterator for GenIterator<G> where G: std::ops::Generator<Return = ()> {
	type Item = G::Yield;

	fn next(&mut self) -> Option<Self::Item> {
		match unsafe { std::ops::Generator::resume(&mut self.0) } {
			std::ops::GeneratorState::Yielded(value) => Some(value),
			std::ops::GeneratorState::Complete(()) => None,
		}
	}
}
