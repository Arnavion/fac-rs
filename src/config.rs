use ::ResultExt;

mod versions {
	#[derive(Clone, Debug, Default, ::serde_derive::Deserialize, ::serde_derive::Serialize, ::derive_struct::getters)]
	pub struct ConfigV1 {
		#[serde(serialize_with = "super::serialize_config_mods")]
		mods: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>,
	}

	impl ConfigV1 {
		pub fn with_mods(self, mods: ::std::collections::HashMap<::factorio_mods_common::ModName, ::factorio_mods_common::ModVersionReq>) -> Self {
			#[cfg_attr(feature = "cargo-clippy", allow(needless_update))]
			ConfigV1 { mods, .. self }
		}
	}
}

#[derive(Clone, Debug, ::serde_derive::Deserialize, ::serde_derive::Serialize)]
#[serde(tag = "version")]
enum StoredConfig {
	V1(versions::ConfigV1),
}

pub type Config = versions::ConfigV1;

impl Config {
	pub fn load(api: &::factorio_mods_local::API) -> ::Result<Self> {
		let user_config_dir = ::appdirs::user_config_dir(Some("fac"), None, false).map_err(|_| "Could not derive path to config directory")?;

		if let Err(err) = ::std::fs::create_dir(&user_config_dir) {
			match err.kind() {
				::std::io::ErrorKind::AlreadyExists => (),
				_ => return Err(err).chain_err(|| format!("Could not create config directory {}", user_config_dir.display())),
			}
		}

		let config_file_path = user_config_dir.join("config.json");
		let config_file_path_displayable = config_file_path.display();

		match ::std::fs::File::open(&config_file_path) {
			Ok(mut file) => {
				let config: StoredConfig = ::serde_json::from_reader(&mut file).chain_err(|| format!("Could not parse JSON file {}", config_file_path_displayable))?;
				let StoredConfig::V1(config) = config;
				Ok(config)
			},

			Err(err) => match err.kind() {
				::std::io::ErrorKind::NotFound => {
					let installed_mods: ::Result<_> =
						api.installed_mods().chain_err(|| "Could not enumerate installed mods")?
						.map(|mod_|
							mod_
							.map(|mod_| (mod_.info().name().clone(), ::factorio_mods_common::ModVersionReq::new(::semver::VersionReq::any())))
							.chain_err(|| "Could not process an installed mod"))
						.collect();
					let installed_mods = installed_mods.chain_err(|| "Could not enumerate installed mods")?;
					Ok(versions::ConfigV1::default().with_mods(installed_mods))
				},

				_ => Err(err).chain_err(|| format!("Could not read config file {}", config_file_path_displayable)),
			},
		}
	}

	pub fn save(self) -> ::Result<()> {
		let user_config_dir = ::appdirs::user_config_dir(Some("fac"), None, false).map_err(|_| "Could not derive path to config directory")?;
		if let Err(err) = ::std::fs::create_dir(&user_config_dir) {
			match err.kind() {
				::std::io::ErrorKind::AlreadyExists => (),
				_ => return Err(err).chain_err(|| format!("Could not create config directory {}", user_config_dir.display())),
			}
		}

		let config_file_path = user_config_dir.join("config.json");
		let config_file_path_displayable = config_file_path.display();

		let mut config_file = ::std::fs::File::create(&config_file_path).chain_err(|| format!("Could not create config file {}", config_file_path_displayable))?;
		::serde_json::to_writer_pretty(&mut config_file, &StoredConfig::V1(self)).chain_err(|| format!("Could not write to config file {}", config_file_path_displayable))
	}
}

fn serialize_config_mods<'a, I, S>(value: I, serializer: S) -> Result<S::Ok, S::Error>
	where I: IntoIterator<Item = (&'a ::factorio_mods_common::ModName, &'a ::factorio_mods_common::ModVersionReq)>, S: ::serde::Serializer {
	let mut map = serializer.serialize_map(None)?;
	for (name, req) in ::itertools::Itertools::sorted_by(value.into_iter(), |&(n1, _), &(n2, _)| n1.cmp(n2)) {
		::serde::ser::SerializeMap::serialize_key(&mut map, name)?;
		::serde::ser::SerializeMap::serialize_value(&mut map, req)?;
	}
	::serde::ser::SerializeMap::end(map)
}
