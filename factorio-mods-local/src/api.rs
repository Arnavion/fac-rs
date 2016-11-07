#[derive(Debug)]
pub struct API {
	write_path: ::std::path::PathBuf,
	config_directory: ::std::path::PathBuf,
	mods_directory: ::std::path::PathBuf,
	mod_status: ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
}

impl API {
	pub fn new() -> ::Result<API> {
		let (write_path, config_directory, mods_directory) =
			FACTORIO_SEARCH_PATHS.iter().filter_map(|search_path| {
				let search_path = ::std::path::Path::new(search_path);

				let config_directory = search_path.join("config");
				let mods_directory = search_path.join("mods");

				if config_directory.is_dir() && mods_directory.is_dir() {
					Some((search_path.into(), config_directory, mods_directory))
				}
				else {
					None
				}
			}).next().ok_or(::ErrorKind::WritePath)?;

		let mod_list_file_path = mods_directory.join("mod-list.json");
		let mod_list_file = ::std::fs::File::open(&mod_list_file_path)?;
		let mod_list: ModList = ::serde_json::from_reader(mod_list_file)?;
		let mod_status = mod_list.mods.into_iter().map(|m| (m.name, m.enabled == "true")).collect();

		Ok(API {
			write_path: write_path,
			config_directory: config_directory,
			mods_directory: mods_directory,
			mod_status: mod_status,
		})
	}

	pub fn mods_directory(&self) -> &::std::path::Path {
		&self.mods_directory
	}

	pub fn installed_mods(&self) -> ::Result<::InstalledModIterator> {
		::InstalledMod::find(&self.mods_directory, None, None, &self.mod_status)
	}

	pub fn user_credentials(&self) -> ::Result<::factorio_mods_common::UserCredentials> {
		let player_data_json_file_path = self.write_path.join("player-data.json");
		let player_data_json_file = ::std::fs::File::open(&player_data_json_file_path)?;
		let player_data: PlayerData = ::serde_json::from_reader(player_data_json_file)?;
		match (player_data.service_username, player_data.service_token) {
			(Some(username), Some(token)) => Ok(::factorio_mods_common::UserCredentials::new(username, token)),
			(Some(username), None) => Err(::ErrorKind::IncompleteUserCredentials(Some(username)).into()),
			_ => Err(::ErrorKind::IncompleteUserCredentials(None).into()),
		}
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

#[derive(Debug, Deserialize)]
struct ModList {
	mods: Vec<ModListMod>,
}

#[derive(Debug, Deserialize)]
struct ModListMod {
	name: ::factorio_mods_common::ModName,
	enabled: String,
}

#[derive(Debug, Deserialize)]
struct PlayerData {
	#[serde(rename(deserialize = "service-username"))]
	service_username: Option<::factorio_mods_common::ServiceUsername>,
	#[serde(rename(deserialize = "service-token"))]
	service_token: Option<::factorio_mods_common::ServiceToken>,
}
