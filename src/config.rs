use crate::{ ErrorExt, ResultExt };

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
	pub fn load(path: Option<std::path::PathBuf>) -> Result<Self, crate::Error>{
		static FACTORIO_INSTALL_SEARCH_PATHS: once_cell::sync::Lazy<Vec<std::path::PathBuf>> =
			once_cell::sync::Lazy::new(|| {
				let mut result = vec![];

				if cfg!(windows) {
					if let Some(path) = std::env::var_os("ProgramW6432") {
						let mut path: std::path::PathBuf = path.into();
						path.push("Steam");
						path.push("steamapps");
						path.push("common");
						path.push("Factorio");
						result.push(path);
					}
					if let Some(path) = std::env::var_os("ProgramFiles") {
						let mut path: std::path::PathBuf = path.into();
						path.push("Steam");
						path.push("steamapps");
						path.push("common");
						path.push("Factorio");
						result.push(path);
					}
				}
				else {
					if let Some(mut path) = dirs::home_dir() {
						path.push(".steam");
						path.push("steam");
						path.push("steamapps");
						path.push("common");
						path.push("Factorio");
						result.push(path);
					}

					if let Some(mut path) = dirs::data_dir() {
						path.push("Steam");
						path.push("steamapps");
						path.push("common");
						path.push("Factorio");
						result.push(path);
					}
				}

				result
			});

		static FACTORIO_USER_SEARCH_PATHS: once_cell::sync::Lazy<Vec<std::path::PathBuf>> =
			once_cell::sync::Lazy::new(|| {
				let mut result = vec![];

				if cfg!(windows) {
					if let Some(mut path) = dirs::data_dir() {
						path.push("Factorio");
						result.push(path);
					}
				}
				else if let Some(mut path) = dirs::home_dir() {
					path.push(".factorio");
					result.push(path);
				}

				result
			});

		let config_file_path =
			if let Some(path) = path {
				if path.iter().count() == 1 {
					let mut user_config_dir = dirs::config_dir().ok_or_else(|| "could not derive path to config directory")?;
					user_config_dir.push("fac");
					user_config_dir.push(path);
					user_config_dir
				}
				else {
					path
				}
			}
			else {
				let mut user_config_dir = dirs::config_dir().ok_or_else(|| "could not derive path to config directory")?;
				user_config_dir.push("fac");
				std::fs::create_dir_all(&user_config_dir).with_context(|| format!("could not create config directory {}", user_config_dir.display()))?;
				user_config_dir.push("config.json");
				user_config_dir
			};

		let config_file_path_displayable = config_file_path.display();

		let (install_directory, user_directory, mods) = match std::fs::File::open(&config_file_path) {
			Ok(mut file) => {
				let config: StoredConfig<'_> =
					serde_json::from_reader(&mut file)
					.with_context(|| format!("could not parse JSON file {}", config_file_path_displayable))?;

				let StoredConfig::V1 { install_directory, user_directory, mods } = config;

				(
					install_directory.map(std::borrow::Cow::into_owned),
					user_directory.map(std::borrow::Cow::into_owned),
					mods.map(std::borrow::Cow::into_owned),
				)
			},

			Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => (None, None, None),

			Err(err) => return Err(err.context(format!("could not read config file {}", config_file_path_displayable))),
		};

		let install_directory =
			install_directory.or_else(|| FACTORIO_INSTALL_SEARCH_PATHS.iter().find_map(|search_path| {
				let search_path = std::path::Path::new(search_path);
				let base_info_file_path = search_path.join("data").join("base").join("info.json");
				if base_info_file_path.is_file() {
					Some(search_path.to_owned())
				}
				else {
					None
				}
			}));

		let user_directory =
			user_directory.or_else(|| FACTORIO_USER_SEARCH_PATHS.iter().find_map(|search_path| {
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
			}));

		Ok(Config {
			path: config_file_path,
			install_directory,
			user_directory,
			mods,
		})
	}

	pub fn save(&self) -> Result<(), crate::Error> {
		let config_file_path_displayable = self.path.display();
		let mut config_file =
			std::fs::File::create(&self.path)
			.with_context(|| format!("could not create config file {}", config_file_path_displayable))?;

		let stored_config = StoredConfig::V1 {
			install_directory: self.install_directory.as_ref().map(AsRef::as_ref).map(std::borrow::Cow::Borrowed),
			user_directory: self.user_directory.as_ref().map(AsRef::as_ref).map(std::borrow::Cow::Borrowed),
			mods: self.mods.as_ref().map(std::borrow::Cow::Borrowed),
		};
		serde_json::to_writer_pretty(&mut config_file, &stored_config)
		.with_context(|| format!("could not write to config file {}", config_file_path_displayable))?;

		Ok(())
	}
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
