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
				return Err(crate::ErrorKind::UnknownModFormat(path.into()).into());
			}

			let zip_file = match std::fs::File::open(&path) {
				Ok(zip_file) => zip_file,
				Err(err) => return Err(crate::ErrorKind::FileIO(path.into(), err).into()),
			};

			let mut zip_file = match zip::ZipArchive::new(zip_file) {
				Ok(zip_file) => zip_file,
				Err(err) => return Err(crate::ErrorKind::Zip(path.into(), err).into()),
			};

			if zip_file.len() == 0 {
				return Err(crate::ErrorKind::EmptyZippedMod(path.into()).into());
			}

			let toplevel = {
				let first_file = match zip_file.by_index(0) {
					Ok(first_file) => first_file,
					Err(err) => return Err(crate::ErrorKind::Zip(path.into(), err).into()),
				};

				first_file.name().split('/').next().unwrap().to_string()
			};

			let info_json_file_path = format!("{}/info.json", toplevel);

			let info_json_file = match zip_file.by_name(&info_json_file_path) {
				Ok(info_json_file) => info_json_file,
				Err(err) => return Err(crate::ErrorKind::Zip(path.into(), err).into()),
			};

			match serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Zipped),
				Err(err) => return Err(crate::ErrorKind::ReadJSONFile(path.into(), err).into()),
			}
		}
		else {
			let info_json_file_path = path.join("info.json");

			let info_json_file = match std::fs::File::open(&info_json_file_path) {
				Ok(info_json_file) => info_json_file,
				Err(err) => match err.kind() {
					std::io::ErrorKind::NotFound => return Err(crate::ErrorKind::UnknownModFormat(info_json_file_path.into()).into()),
					_ => return Err(crate::ErrorKind::FileIO(info_json_file_path.into(), err).into()),
				},
			};

			match serde_json::from_reader(info_json_file) {
				Ok(info) => (info, InstalledModType::Unpacked),
				Err(err) => return Err(crate::ErrorKind::ReadJSONFile(info_json_file_path.into(), err).into()),
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

	Ok(directory_entries
		.filter_map(move |directory_entry| {
			let directory_entry = match directory_entry {
				Ok(directory_entry) => directory_entry,
				Err(err) => return Some(Err(err.into())),
			};

			let path = directory_entry.path();

			let matches = path.file_name().map_or(false, |filename| matcher.is_match(filename));
			if !matches {
				return None;
			}

			let installed_mod = match InstalledMod::parse(path) {
				Ok(installed_mod) => installed_mod,

				Err(err) => match err.kind() {
					crate::ErrorKind::UnknownModFormat(_) => return None,
					_ => return Some(Err(err)),
				},
			};

			if let Some(version) = &version {
				if version != &installed_mod.info.version {
					return None;
				}
			}

			Some(Ok(installed_mod))
		}))
}

lazy_static::lazy_static! {
	static ref DEFAULT_GAME_VERSION: factorio_mods_common::ModVersionReq = factorio_mods_common::ModVersionReq("0.12".parse().unwrap());
	static ref DEFAULT_DEPENDENCIES: Vec<factorio_mods_common::Dependency> = vec![factorio_mods_common::Dependency {
		name: factorio_mods_common::ModName("base".to_string()),
		version: factorio_mods_common::ModVersionReq(semver::VersionReq::any()),
		kind: package::DependencyKind::Required,
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
