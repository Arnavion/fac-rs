#![allow(
	clippy::identity_op,
	clippy::single_match_else,
)]

use failure::{ Fail, ResultExt };

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(tag = "version")]
enum StoredConfig<'a> {
	V1 {
		install_directory: Option<std::borrow::Cow<'a, std::path::Path>>,
		user_directory: Option<std::borrow::Cow<'a, std::path::Path>>,
		#[serde(serialize_with = "serialize_config_mods")]
		mods: Option<std::borrow::Cow<'a, std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>>>,
	},
}

#[derive(Debug)]
pub(crate) struct Config {
	path: std::path::PathBuf,

	pub install_directory: Option<std::path::PathBuf>,
	pub user_directory: Option<std::path::PathBuf>,
	pub mods: Option<std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>>,
}

impl Config {
	pub fn load(path: Option<std::path::PathBuf>) -> Result<Self, failure::Error>{
		let config_file_path = match path {
			Some(path) => path,
			None => {
				let user_config_dir = appdirs::user_config_dir(Some("fac"), None, false).map_err(|()| failure::err_msg("Could not derive path to config directory"))?;

				if let Err(err) = std::fs::create_dir(&user_config_dir) {
					match err.kind() {
						std::io::ErrorKind::AlreadyExists => (),
						_ => return Err(err.context(format!("Could not create config directory {}", user_config_dir.display())).into()),
					}
				}

				user_config_dir.join("config.json")
			},
		};

		let config_file_path_displayable = config_file_path.display();

		let (install_directory, user_directory, mods) = match std::fs::File::open(&config_file_path) {
			Ok(mut file) => {
				let config: StoredConfig<'_> =
					serde_json::from_reader(&mut file)
					.with_context(|_| format!("Could not parse JSON file {}", config_file_path_displayable))?;

				let StoredConfig::V1 { install_directory, user_directory, mods } = config;

				(
					install_directory.map(std::borrow::Cow::into_owned),
					user_directory.map(std::borrow::Cow::into_owned),
					mods.map(std::borrow::Cow::into_owned),
				)
			},

			Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => (None, None, None),

			Err(err) => return Err(err.context(format!("Could not read config file {}", config_file_path_displayable)).into()),
		};

		let install_directory =
			if let Some(install_directory) = install_directory {
				Some(install_directory)
			}
			else {
				FACTORIO_SEARCH_PATHS.iter().find_map(|search_path| {
					let search_path = std::path::Path::new(search_path);
					let base_info_file_path = search_path.join("data").join("base").join("info.json");
					if base_info_file_path.is_file() {
						Some(search_path.to_owned())
					}
					else {
						None
					}
				})
			};

		let user_directory =
			if let Some(user_directory) = user_directory {
				Some(user_directory)
			}
			else {
				FACTORIO_SEARCH_PATHS.iter().find_map(|search_path| {
					let search_path = std::path::Path::new(search_path);

					let mods_directory = search_path.join("mods");
					let mod_list_file_path = mods_directory.join("mod-list.json");
					let player_data_json_file_path = search_path.join("player-data.json");

					if mod_list_file_path.is_file() && player_data_json_file_path.is_file() {
						Some(search_path.to_owned())
					}
					else {
						None
					}
				})
			};

		Ok(Config {
			path: config_file_path,
			install_directory,
			user_directory,
			mods,
		})
	}

	pub fn save(&self) -> Result<(), failure::Error> {
		let config_file_path_displayable = self.path.display();
		let mut config_file =
			std::fs::File::create(&self.path)
			.with_context(|_| format!("Could not create config file {}", config_file_path_displayable))?;

		let stored_config = StoredConfig::V1 {
			install_directory: self.install_directory.as_ref().map(AsRef::as_ref).map(std::borrow::Cow::Borrowed),
			user_directory: self.user_directory.as_ref().map(AsRef::as_ref).map(std::borrow::Cow::Borrowed),
			mods: self.mods.as_ref().map(std::borrow::Cow::Borrowed),
		};
		serde_json::to_writer_pretty(&mut config_file, &stored_config)
		.with_context(|_| format!("Could not write to config file {}", config_file_path_displayable))?;

		Ok(())
	}
}

lazy_static::lazy_static! {
	static ref FACTORIO_SEARCH_PATHS: Vec<std::path::PathBuf> = {
		let mut result = vec![];

		if let Ok(current_dir) = std::env::current_dir() {
			result.push(current_dir.clone());

			let current_directory = current_dir.join("factorio");
			if current_directory.is_dir() {
				result.push(current_directory);
			}

			if let Some(parent_dir) = current_dir.parent() {
				result.push(std::path::PathBuf::from(parent_dir));

				let parent_directory = current_dir.join("factorio");
				if parent_directory.is_dir() {
					result.push(parent_directory);
				}
			}
		}

		if let Ok(user_data_dir) = appdirs::user_data_dir(Some("factorio"), None, false) {
			if user_data_dir.is_dir() {
				result.push(user_data_dir);
			}
		}

		if let Ok(user_data_dir) = appdirs::user_data_dir(Some("Steam"), None, false) {
			let mut steam_directory = user_data_dir;
			steam_directory.push("steamapps");
			steam_directory.push("common");
			steam_directory.push("Factorio");
			if steam_directory.is_dir() {
				result.push(steam_directory);
			}
		}

		if cfg!(windows) {
			if let Some(appdata) = std::env::var_os("APPDATA") {
				let appdata_directory = std::path::Path::new(&appdata).join("factorio");
				if appdata_directory.is_dir() {
					result.push(appdata_directory);
				}
			}

			result.push(std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Factorio"));
		}

		if cfg!(target_os = "linux") {
			if let Some(home) = std::env::var_os("HOME") {
				let home_directory = std::path::Path::new(&home);
				result.push(home_directory.join("factorio"));
				result.push(home_directory.join(".factorio"));
			}
		}

		result
	};
}

fn serialize_config_mods<S>(
	value: &Option<std::borrow::Cow<'_, std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>>>,
	serializer: S,
) -> Result<S::Ok, S::Error> where S: serde::Serializer {
	if let Some(value) = value {
		let mut map = serializer.serialize_map(None)?;
		for (name, req) in itertools::Itertools::sorted_by(value.iter(), |&(n1, _), &(n2, _)| n1.cmp(n2)) {
			serde::ser::SerializeMap::serialize_key(&mut map, name)?;
			serde::ser::SerializeMap::serialize_value(&mut map, req)?;
		}
		serde::ser::SerializeMap::end(map)
	}
	else {
		serializer.serialize_none()
	}
}
