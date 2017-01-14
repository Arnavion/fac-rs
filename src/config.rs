mod versions {
	#[derive(Clone, Debug, Deserialize, Serialize, getters)]
	pub struct ConfigV1 {
		#[serde(serialize_with = "super::serialize_config_mods")]
		mods: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
	}

	impl ConfigV1 {
		pub fn with_mods(self, mods: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>) -> ConfigV1 {
			ConfigV1 { mods: mods, .. self }
		}
	}

	lazy_static! {
		pub static ref DEFAULT_CONFIG_V1: ConfigV1 = ConfigV1 { mods: Default::default() };
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
				::std::io::ErrorKind::NotFound =>
					versions::DEFAULT_CONFIG_V1.clone().with_mods(api.installed_mods().unwrap().map(|installed_mod| {
						let installed_mod = installed_mod.unwrap();
						(installed_mod.info().name().clone(), ::factorio_mods_common::ModVersionReq::new(::semver::VersionReq::any()))
					}).collect()),

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

fn serialize_config_mods<'a, I, S>(value: I, serializer: &mut S) -> Result<(), S::Error>
	where I: IntoIterator<Item = (&'a ::factorio_mods_common::ModName, &'a ::factorio_mods_common::ModVersionReq)>, S: ::serde::Serializer {
	let mut state = serializer.serialize_map(None)?;
	for (name, req) in ::itertools::Itertools::sorted_by(value.into_iter(), |&(n1, _), &(n2, _)| n1.cmp(n2)) {
		serializer.serialize_map_key(&mut state, name)?;
		serializer.serialize_map_value(&mut state, req)?;
	}
	serializer.serialize_map_end(state)
}
