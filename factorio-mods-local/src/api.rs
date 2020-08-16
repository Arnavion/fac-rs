/// Entry-point to the local Factorio API
#[derive(Debug)]
pub struct API {
	game_version: factorio_mods_common::ReleaseVersion,
	mods_directory: std::path::PathBuf,
	mod_list_file_path: std::path::PathBuf,
	player_data_json_file_path: std::path::PathBuf,
}

impl API {
	/// Constructs an API object.
	pub fn new(install_directory: &std::path::Path, user_directory: &std::path::Path) -> Result<Self, crate::Error> {
		let game_version = {
			let mut base_info_file_path = install_directory.join("data");
			base_info_file_path.push("base");
			base_info_file_path.push("info.json");
			if !base_info_file_path.is_file() {
				return Err(crate::ErrorKind::InstallDirectoryNotFound.into());
			}

			let base_info_file = match std::fs::File::open(&base_info_file_path) {
				Ok(base_info_file) => base_info_file,
				Err(err) => return Err(crate::ErrorKind::Io(base_info_file_path, err).into()),
			};
			let base_info: BaseInfo = serde_json::from_reader(base_info_file).map_err(|err| crate::ErrorKind::ReadJSONFile(base_info_file_path, err))?;
			base_info.version
		};

		let (mods_directory, mod_list_file_path, player_data_json_file_path) = {
			let mods_directory = user_directory.join("mods");
			let mod_list_file_path = mods_directory.join("mod-list.json");
			let player_data_json_file_path = user_directory.join("player-data.json");
			if mod_list_file_path.is_file() && player_data_json_file_path.is_file() {
				(mods_directory, mod_list_file_path, player_data_json_file_path)
			}
			else {
				return Err(crate::ErrorKind::UserDirectoryNotFound.into());
			}
		};

		Ok(API {
			game_version,
			mods_directory,
			mod_list_file_path,
			player_data_json_file_path,
		})
	}

	/// Returns the game version.
	pub fn game_version(&self) -> &factorio_mods_common::ReleaseVersion {
		&self.game_version
	}

	/// Returns the directory where mods should be installed.
	pub fn mods_directory(&self) -> &std::path::Path {
		&self.mods_directory
	}

	/// Returns an iterator over all the locally installed mods, matching the given name pattern if any.
	pub fn installed_mods(&self) -> Result<impl Iterator<Item = Result<crate::InstalledMod, crate::Error>> + 'static, crate::Error> {
		crate::installed_mod::find(&self.mods_directory, None, None)
	}

	/// Fetches the locally saved user credentials, if any.
	pub fn user_credentials(&self) -> Result<factorio_mods_common::UserCredentials, crate::Error> {
		let player_data_json_file_path = &self.player_data_json_file_path;

		let player_data_json_file = match std::fs::File::open(player_data_json_file_path) {
			Ok(player_data_json_file) => player_data_json_file,
			Err(err) => return Err(crate::ErrorKind::Io(player_data_json_file_path.into(), err).into()),
		};

		let PlayerData { service_username, service_token } =
			serde_json::from_reader(player_data_json_file).map_err(|err| crate::ErrorKind::ReadJSONFile(player_data_json_file_path.into(), err))?;

		if service_username.0.is_empty() {
			return Err(crate::ErrorKind::IncompleteUserCredentials(None).into());
		}

		if service_token.0.is_empty() {
			return Err(crate::ErrorKind::IncompleteUserCredentials(Some(service_username)).into());
		}

		Ok(factorio_mods_common::UserCredentials { username: service_username, token: service_token })
	}

	/// Saves the given user credentials to `player-data.json`
	pub fn save_user_credentials(&self, user_credentials: factorio_mods_common::UserCredentials) -> Result<(), crate::Error> {
		let player_data_json_file_path = &self.player_data_json_file_path;

		let mut player_data: serde_json::Map<_, _> = {
			let player_data_json_file = match std::fs::File::open(player_data_json_file_path) {
				Ok(player_data_json_file) => player_data_json_file,
				Err(err) => return Err(crate::ErrorKind::Io(player_data_json_file_path.into(), err).into()),
			};

			serde_json::from_reader(player_data_json_file).map_err(|err| crate::ErrorKind::ReadJSONFile(player_data_json_file_path.into(), err))?
		};

		player_data.insert("service-username".to_string(), serde_json::Value::String(user_credentials.username.0));
		player_data.insert("service-token".to_string(), serde_json::Value::String(user_credentials.token.0));

		let player_data = player_data;

		let mut player_data_json_file = match std::fs::File::create(player_data_json_file_path) {
			Ok(player_data_json_file) => player_data_json_file,
			Err(err) => return Err(crate::ErrorKind::Io(player_data_json_file_path.into(), err).into()),
		};

		serde_json::to_writer_pretty(&mut player_data_json_file, &player_data).map_err(|err| crate::ErrorKind::WriteJSONFile(player_data_json_file_path.into(), err).into())
	}

	/// Returns a map of installed mod name to its enabled / disabled status in `mod-list.json`
	pub fn mods_status(&self) -> Result<std::collections::HashMap<factorio_mods_common::ModName, bool>, crate::Error> {
		let mod_list = self.load_mod_list()?;
		Ok(mod_list.mods.into_iter().map(|m| (m.name.into_owned(), m.enabled)).collect())
	}

	/// Marks the given locally installed mods as enabled or disabled in `mod-list.json`
	pub fn set_enabled<'a, I>(&self, installed_mods: I, enabled: bool) -> Result<(), crate::Error> where I: IntoIterator<Item = &'a crate::InstalledMod> {
		let mod_list = self.load_mod_list()?;
		let mut mods_status: std::collections::HashMap<_, _> = mod_list.mods.into_iter().map(|m| (m.name, m.enabled)).collect();

		for installed_mod in installed_mods {
			mods_status.insert(std::borrow::Cow::Borrowed(&installed_mod.info.name), enabled);
		}

		let mod_list_file_path = &self.mod_list_file_path;
		let mut mod_list_file = match std::fs::File::create(mod_list_file_path) {
			Ok(mod_list_file) => mod_list_file,
			Err(err) => return Err(crate::ErrorKind::Io(mod_list_file_path.into(), err).into()),
		};

		let mut mods: Vec<_> =
			mods_status.into_iter()
			.map(|(name, enabled)| ModListMod { name, enabled })
			.collect();
		mods.sort_by(|mod1, mod2| mod1.name.cmp(&mod2.name));

		let mod_list = ModList { mods };
		serde_json::to_writer_pretty(&mut mod_list_file, &mod_list).map_err(|err| crate::ErrorKind::WriteJSONFile(mod_list_file_path.into(), err).into())
	}

	fn load_mod_list(&self) -> Result<ModList<'static>, crate::Error> {
		let mod_list_file_path = &self.mod_list_file_path;
		let mod_list_file = match std::fs::File::open(mod_list_file_path) {
			Ok(mod_list_file) => mod_list_file,
			Err(err) => return Err(crate::ErrorKind::Io(mod_list_file_path.into(), err).into()),
		};
		Ok(serde_json::from_reader(mod_list_file).map_err(|err| crate::ErrorKind::ReadJSONFile(mod_list_file_path.into(), err))?)
	}
}

/// Represents the contents of `mod-list.json`
#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
struct ModList<'a> {
	mods: Vec<ModListMod<'a>>,
}

/// A mod entry in the mod list
#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
struct ModListMod<'a> {
	name: std::borrow::Cow<'a, factorio_mods_common::ModName>,

	#[serde(deserialize_with = "deserialize_mod_list_mod_enabled")]
	enabled: bool,
}

/// Represents the contents of `base/info.json`
#[derive(Debug, serde_derive::Deserialize)]
struct BaseInfo {
	version: factorio_mods_common::ReleaseVersion,
}

/// Represents the contents of `player-data.json`
#[derive(Debug, serde_derive::Deserialize)]
struct PlayerData {
	// A clean install of Factorio defaults these fields to the empty string if the user has not logged in,
	// so coerce them to the empty string even if they don't exist and treat empty string as if they don't exist.
	#[serde(rename(deserialize = "service-username"), default)]
	service_username: factorio_mods_common::ServiceUsername,

	#[serde(rename(deserialize = "service-token"), default)]
	service_token: factorio_mods_common::ServiceToken,
}

/// Deserializes the `enabled` field of a mod in `mod-list.json`, which can be a JSON string or a JSON boolean.
fn deserialize_mod_list_mod_enabled<'de, D>(deserializer: D) -> Result<bool, D::Error>
	where D: serde::Deserializer<'de> {

	struct Visitor;

	impl serde::de::Visitor<'_> for Visitor {
		type Value = bool;

		fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, r#""true" or "false" or true or false"#)
		}

		fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: serde::de::Error {
			Ok(v)
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
			v.parse().map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self))
		}
	}

	deserializer.deserialize_any(Visitor)
}
