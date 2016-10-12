use installed_mod;
use types;

#[derive(Debug)]
pub struct Config {
	config_directory:  ::std::path::PathBuf,
	mods_directory: ::std::path::PathBuf,
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

impl Config {
	pub fn new() -> Result<Config, types::LocalError> {
		let (config_directory, mods_directory) =
			try!(FACTORIO_SEARCH_PATHS.iter().filter_map(|search_path| {
				let search_path = ::std::path::Path::new(search_path);

				let config_directory = search_path.join("config");
				let mods_directory = search_path.join("mods");

				if config_directory.is_dir() && mods_directory.is_dir() {
					Some((config_directory, mods_directory))
				}
				else {
					None
				}
			}).next().ok_or_else(types::LocalError::write_path));

		Ok(Config {
			config_directory: config_directory,
			mods_directory: mods_directory,
		})
	}
}

#[derive(Debug)]
pub struct Manager {
	config: Config,
	mod_status: ::std::collections::HashMap<::factorio_mods_common::ModName, bool>,
}

impl Manager {
	pub fn new(config: Config) -> Result<Manager, types::LocalError> {
		let mod_list_file_path = config.mods_directory.join("mod-list.json");
		let mod_list_file = try!(::std::fs::File::open(&mod_list_file_path).map_err(|_| types::LocalError::mod_list(mod_list_file_path.clone())));
		let mut mod_list: ModList = try!(::serde_json::from_reader(mod_list_file).map_err(|_| types::LocalError::mod_list(mod_list_file_path)));
		let mod_status = mod_list.mods.drain(..).map(|m| (m.name, m.enabled == "true")).collect();

		Ok(Manager {
			config: config,
			mod_status: mod_status,
		})
	}

	pub fn installed_mods(&self) -> Result<installed_mod::InstalledModIterator, types::LocalError> {
		installed_mod::InstalledMod::find(&self.config.mods_directory, None, None, &self.mod_status)
	}
}

make_deserializable!(struct ModList {
	mods: Vec<ModListMod>,
});

make_deserializable!(struct ModListMod {
	name: ::factorio_mods_common::ModName,
	enabled: String,
});
