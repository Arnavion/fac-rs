mod versions {
	#[derive(Clone, Debug, Deserialize, Serialize, getters)]
	pub struct ConfigV1 {
	}

	lazy_static! {
		pub static ref DEFAULT_CONFIG_V1: ConfigV1 = ConfigV1 { };
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum StoredConfig {
	V1(versions::ConfigV1),
}

pub type Config = versions::ConfigV1;

impl Config {
	pub fn load(api: &::factorio_mods_local::API) -> Config {
		let user_config_dir = ::appdirs::user_config_dir(Some("fac"), None, false).unwrap();
		if let Err(err) = ::std::fs::create_dir(&user_config_dir) {
			match err.kind() {
				::std::io::ErrorKind::AlreadyExists => { },
				_ => panic!(err),
			}
		}

		let config_file_path = user_config_dir.join("config.json");
		match ::std::fs::File::open(config_file_path) {
			Ok(mut file) => {
				let config: StoredConfig = ::serde_json::from_reader(&mut file).unwrap();
				let StoredConfig::V1(config) = config;
				config
			},

			Err(err) => match err.kind() {
				::std::io::ErrorKind::NotFound => versions::DEFAULT_CONFIG_V1.clone(),

				_ => panic!(err),
			},
		}
	}

	pub fn save(self) {
		let user_config_dir = ::appdirs::user_config_dir(Some("fac"), None, false).unwrap();
		if let Err(err) = ::std::fs::create_dir(&user_config_dir) {
			match err.kind() {
				::std::io::ErrorKind::AlreadyExists => { },
				_ => panic!(err),
			}
		}

		let config_file_path = user_config_dir.join("config.json");
		let mut config_file = ::std::fs::File::create(config_file_path).unwrap();
		::serde_json::to_writer_pretty(&mut config_file, &StoredConfig::V1(self)).unwrap()
	}
}
