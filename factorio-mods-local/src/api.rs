/// Entry-point to the local Factorio API
#[derive(Debug)]
pub struct API {
	game_version: ::factorio_mods_common::ReleaseVersion,
	write_path: ::std::path::PathBuf,
	mods_directory: ::std::path::PathBuf,
	mod_status: ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
}

impl API {
	/// Constructs an API object. Tries to detect the local Factorio install in some pre-defined locations.
	pub fn new() -> ::Result<API> {
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
				Err(err) => bail!(::ErrorKind::IO(base_info_file_path, err)),
			};
			let base_info: BaseInfo = ::serde_json::from_reader(base_info_file).map_err(|err| ::ErrorKind::JSON(base_info_file_path, err))?;
			base_info.version
		};

		let (write_path, mods_directory, mod_list_file_path) =
			FACTORIO_SEARCH_PATHS.iter().filter_map(|search_path| {
				let search_path = ::std::path::Path::new(search_path);

				let mods_directory = search_path.join("mods");
				let mod_list_file_path = mods_directory.join("mod-list.json");

				if mod_list_file_path.is_file() {
					Some((search_path.into(), mods_directory, mod_list_file_path))
				}
				else {
					None
				}
			}).next().ok_or(::ErrorKind::WritePath)?;

		let mod_status = {
			let mod_list_file = match ::std::fs::File::open(&mod_list_file_path) {
				Ok(mod_list_file) => mod_list_file,
				Err(err) => bail!(::ErrorKind::IO(mod_list_file_path, err)),
			};
			let mod_list: ModList = ::serde_json::from_reader(mod_list_file).map_err(|err| ::ErrorKind::JSON(mod_list_file_path, err))?;
			mod_list.mods.into_iter().map(|m| (m.name, m.enabled == "true")).collect()
		};

		Ok(API {
			game_version: game_version,
			write_path: write_path,
			mods_directory: mods_directory,
			mod_status: mod_status,
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

	/// Returns an iterator over all the locally installed mods.
	pub fn installed_mods<'a>(&'a self) -> ::Result<impl Iterator<Item = ::Result<::InstalledMod>> + 'a> {
		::installed_mod::find(&self.mods_directory, None, None, &self.mod_status)
	}

	/// Fetches the locally saved user credentials, if any.
	pub fn user_credentials(&self) -> ::Result<::factorio_mods_common::UserCredentials> {
		let player_data_json_file_path = self.write_path.join("player-data.json");
		let player_data_json_file = match ::std::fs::File::open(&player_data_json_file_path) {
			Ok(player_data_json_file) => player_data_json_file,
			Err(err) => bail!(::ErrorKind::IO(player_data_json_file_path, err)),
		};
		let player_data: PlayerData = ::serde_json::from_reader(player_data_json_file).map_err(|err| ::ErrorKind::JSON(player_data_json_file_path, err))?;
		Ok(match (player_data.service_username, player_data.service_token) {
			(Some(username), Some(token)) => ::factorio_mods_common::UserCredentials::new(username, token),
			(username, _) => bail!(::ErrorKind::IncompleteUserCredentials(username)),
		})
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
			steam_directory.push("SteamApps");
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

		result
	};
}

/// Represents the contents of `mod-list.json`
#[derive(Debug, Deserialize)]
struct ModList {
	mods: Vec<ModListMod>,
}

/// A mod entry in the mod list
#[derive(Debug, Deserialize)]
struct ModListMod {
	name: ::factorio_mods_common::ModName,
	enabled: String,
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
