/// Entry-point to the local Factorio API
#[derive(Debug)]
pub struct API {
	game_version: ::factorio_mods_common::ReleaseVersion,
	mods_directory: ::std::path::PathBuf,
	mod_list_file_path: ::std::path::PathBuf,
	player_data_json_file_path: ::std::path::PathBuf,
}

impl API {
	/// Constructs an API object. Tries to detect the local Factorio install in some pre-defined locations.
	pub fn new() -> ::Result<Self> {
		let base_info_file_path = FACTORIO_SEARCH_PATHS.iter().filter_map(|search_path| {
			let search_path = ::std::path::Path::new(search_path);
			let base_info_file_path = search_path.join("data").join("base").join("info.json");
			if base_info_file_path.is_file() {
				Some(base_info_file_path)
			}
			else {
				None
			}
		}).next().ok_or(::ErrorKind::DataPath)?;

		let game_version = {
			let base_info_file = match ::std::fs::File::open(&base_info_file_path) {
				Ok(base_info_file) => base_info_file,
				Err(err) => bail!(::ErrorKind::FileIO(base_info_file_path, err)),
			};
			let base_info: BaseInfo = ::serde_json::from_reader(base_info_file).map_err(|err| ::ErrorKind::ReadJSONFile(base_info_file_path, err))?;
			base_info.version
		};

		let (mods_directory, mod_list_file_path, player_data_json_file_path) =
			FACTORIO_SEARCH_PATHS.iter().filter_map(|search_path| {
				let search_path = ::std::path::Path::new(search_path);

				let mods_directory = search_path.join("mods");
				let mod_list_file_path = mods_directory.join("mod-list.json");
				let player_data_json_file_path = search_path.join("player-data.json");

				if mod_list_file_path.is_file() && player_data_json_file_path.is_file() {
					Some((mods_directory, mod_list_file_path, player_data_json_file_path))
				}
				else {
					None
				}
			}).next().ok_or(::ErrorKind::WritePath)?;

		Ok(API {
			game_version,
			mods_directory,
			mod_list_file_path,
			player_data_json_file_path,
		})
	}

	/// Returns the game version.
	pub fn game_version(&self) -> &::factorio_mods_common::ReleaseVersion {
		&self.game_version
	}

	/// Returns the directory where mods should be installed.
	pub fn mods_directory(&self) -> &::std::path::Path {
		&self.mods_directory
	}

	/// Returns an iterator over all the locally installed mods, matching the given name pattern if any.
	pub fn installed_mods<'a>(&'a self) -> ::Result<impl Iterator<Item = ::Result<::InstalledMod>> + 'a> {
		let mod_status = self.load_mod_status()?;
		::installed_mod::find(&self.mods_directory, None, None, mod_status)
	}

	/// Fetches the locally saved user credentials, if any.
	pub fn user_credentials(&self) -> ::Result<::factorio_mods_common::UserCredentials> {
		let player_data_json_file_path = &self.player_data_json_file_path;

		let player_data_json_file = match ::std::fs::File::open(player_data_json_file_path) {
			Ok(player_data_json_file) => player_data_json_file,
			Err(err) => bail!(::ErrorKind::FileIO(player_data_json_file_path.into(), err)),
		};

		let player_data: PlayerData = ::serde_json::from_reader(player_data_json_file).map_err(|err| ::ErrorKind::ReadJSONFile(player_data_json_file_path.into(), err))?;

		Ok(match (player_data.service_username, player_data.service_token) {
			(Some(username), Some(token)) => ::factorio_mods_common::UserCredentials::new(username, token),
			(username, _) => bail!(::ErrorKind::IncompleteUserCredentials(username)),
		})
	}

	/// Saves the given user credentials to `player-data.json`
	pub fn save_user_credentials(&self, user_credentials: &::factorio_mods_common::UserCredentials) -> ::Result<()> {
		let player_data_json_file_path = &self.player_data_json_file_path;

		let mut player_data: ::serde_json::Map<_, _> = {
			let player_data_json_file = match ::std::fs::File::open(player_data_json_file_path) {
				Ok(player_data_json_file) => player_data_json_file,
				Err(err) => bail!(::ErrorKind::FileIO(player_data_json_file_path.into(), err)),
			};

			::serde_json::from_reader(player_data_json_file).map_err(|err| ::ErrorKind::ReadJSONFile(player_data_json_file_path.into(), err))?
		};

		player_data.insert("service-username".to_string(), ::serde_json::Value::String(user_credentials.username().to_string()));
		player_data.insert("service-token".to_string(), ::serde_json::Value::String(user_credentials.token().to_string()));

		let player_data = player_data;

		let mut player_data_json_file = match ::std::fs::File::create(player_data_json_file_path) {
			Ok(player_data_json_file) => player_data_json_file,
			Err(err) => bail!(::ErrorKind::FileIO(player_data_json_file_path.into(), err)),
		};

		::serde_json::to_writer_pretty(&mut player_data_json_file, &player_data).map_err(|err| ::ErrorKind::WriteJSONFile(player_data_json_file_path.into(), err).into())
	}

	/// Marks the given locally installed mods as enabled or disabled in `mod-list.json`
	pub fn set_enabled<'a, I>(&self, installed_mods: I, enabled: bool) -> ::Result<()> where I: IntoIterator<Item = &'a ::InstalledMod> {
		let mut mod_status = self.load_mod_status()?;

		for installed_mod in installed_mods {
			mod_status.insert(installed_mod.info().name().clone(), enabled);
		}

		self.save_mod_status(&mod_status)
	}

	fn load_mod_status(&self) -> ::Result<::std::collections::HashMap<::factorio_mods_common::ModName, bool>> {
		let mod_list_file_path = &self.mod_list_file_path;
		let mod_list_file = match ::std::fs::File::open(mod_list_file_path) {
			Ok(mod_list_file) => mod_list_file,
			Err(err) => bail!(::ErrorKind::FileIO(mod_list_file_path.into(), err)),
		};
		let mod_list: ModList = ::serde_json::from_reader(mod_list_file).map_err(|err| ::ErrorKind::ReadJSONFile(mod_list_file_path.into(), err))?;
		Ok(mod_list.mods.into_iter().map(|m| (m.name, m.enabled)).collect())
	}

	fn save_mod_status(&self, mod_status: &::std::collections::HashMap<::factorio_mods_common::ModName, bool>) -> ::Result<()> {
		let mod_list_file_path = &self.mod_list_file_path;
		let mut mod_list_file = match ::std::fs::File::create(mod_list_file_path) {
			Ok(mod_list_file) => mod_list_file,
			Err(err) => bail!(::ErrorKind::FileIO(mod_list_file_path.into(), err)),
		};

		let mut mods: Vec<_> =
			mod_status.into_iter()
			.map(|(name, &enabled)| ModListMod { name: name.clone(), enabled })
			.collect();
		mods.sort_by(|mod1, mod2| mod1.name.cmp(&mod2.name));

		let mod_list = ModList { mods };
		::serde_json::to_writer_pretty(&mut mod_list_file, &mod_list).map_err(|err| ::ErrorKind::WriteJSONFile(mod_list_file_path.into(), err).into())
	}
}

lazy_static! {
	static ref FACTORIO_SEARCH_PATHS: Vec<::std::path::PathBuf> = {
		let mut result = vec![];

		if let Ok(current_dir) = ::std::env::current_dir() {
			result.push(current_dir.clone());

			let current_directory = current_dir.join("factorio");
			if current_directory.is_dir() {
				result.push(current_directory);
			}

			if let Some(parent_dir) = current_dir.parent() {
				result.push(::std::path::PathBuf::from(parent_dir));

				let parent_directory = current_dir.join("factorio");
				if parent_directory.is_dir() {
					result.push(parent_directory);
				}
			}
		}

		if let Ok(user_data_dir) = ::appdirs::user_data_dir(Some("factorio"), None, false) {
			if user_data_dir.is_dir() {
				result.push(user_data_dir);
			}
		}

		if let Ok(user_data_dir) = ::appdirs::user_data_dir(Some("Steam"), None, false) {
			let mut steam_directory = user_data_dir;
			steam_directory.push("steamapps");
			steam_directory.push("common");
			steam_directory.push("Factorio");
			if steam_directory.is_dir() {
				result.push(steam_directory);
			}
		}

		if cfg!(windows) {
			if let Some(appdata) = ::std::env::var_os("APPDATA") {
				let appdata_directory = ::std::path::Path::new(&appdata).join("factorio");
				if appdata_directory.is_dir() {
					result.push(appdata_directory);
				}
			}

			result.push(::std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Factorio"));
		}

		if cfg!(target_os = "linux") {
			if let Some(home) = ::std::env::var_os("HOME") {
				let home_directory = ::std::path::Path::new(&home);
				result.push(home_directory.join("factorio"));
				result.push(home_directory.join(".factorio"));
			}
		}

		result
	};
}

/// Represents the contents of `mod-list.json`
#[derive(Debug, Deserialize, Serialize)]
struct ModList {
	mods: Vec<ModListMod>,
}

/// A mod entry in the mod list
#[derive(Debug, Deserialize, Serialize)]
struct ModListMod {
	name: ::factorio_mods_common::ModName,

	#[serde(deserialize_with = "deserialize_mod_list_mod_enabled")]
	enabled: bool,
}

/// Represents the contents of `base/info.json`
#[derive(Debug, Deserialize)]
struct BaseInfo {
	version: ::factorio_mods_common::ReleaseVersion,
}

/// Represents the contents of `player-data.json`
#[derive(Debug, Deserialize)]
struct PlayerData {
	#[serde(rename(deserialize = "service-username"))]
	service_username: Option<::factorio_mods_common::ServiceUsername>,
	#[serde(rename(deserialize = "service-token"))]
	service_token: Option<::factorio_mods_common::ServiceToken>,
}

/// Deserializes the `enabled` field of a mod in `mod-list.json`, which can be a JSON string or a JSON boolean.
pub fn deserialize_mod_list_mod_enabled<'de, D>(deserializer: D) -> Result<bool, D::Error>
	where D: ::serde::Deserializer<'de> {

	struct Visitor;

	impl<'de> ::serde::de::Visitor<'de> for Visitor {
		type Value = bool;

		fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
			write!(f, r#""true" or "false" or true or false"#)
		}

		fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: ::serde::de::Error {
			Ok(v)
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: ::serde::de::Error {
			v.parse().map_err(|_| ::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(v), &self))
		}
	}

	deserializer.deserialize_any(Visitor)
}
