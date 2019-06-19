#![allow(
	clippy::identity_op,
	clippy::single_match_else,
)]

use failure::{ Fail, ResultExt };

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(tag = "version")]
enum StoredConfig<'a> {
	V1 {
		#[serde(serialize_with = "serialize_config_mods")]
		mods: std::borrow::Cow<'a, std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>>,
	},
}

#[derive(Debug)]
pub struct Config {
	path: std::path::PathBuf,
	pub mods: std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
}

impl Config {
	pub fn load(api: &factorio_mods_local::API, path: Option<std::path::PathBuf>) -> Result<Self, failure::Error>{
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

		let (mods,) = match std::fs::File::open(&config_file_path) {
			Ok(mut file) => {
				let config: StoredConfig<'_> =
					serde_json::from_reader(&mut file)
					.with_context(|_| format!("Could not parse JSON file {}", config_file_path_displayable))?;
				let StoredConfig::V1 { mods } = config;
				(mods.into_owned(),)
			},

			Err(err) => match err.kind() {
				std::io::ErrorKind::NotFound => {
					// Default config is the list of all currently installed mods with a * requirement
					let installed_mods: Result<_, failure::Error> =
						api.installed_mods().context("Could not enumerate installed mods")?
						.map(|mod_| Ok(
							mod_
							.map(|mod_| (mod_.info.name, factorio_mods_common::ModVersionReq(semver::VersionReq::any())))
							.context("Could not process an installed mod")?))
						.collect();
					let mods = installed_mods.context("Could not enumerate installed mods")?;
					(mods,)
				},

				_ => return Err(err.context(format!("Could not read config file {}", config_file_path_displayable)).into()),
			},
		};

		Ok(Config {
			path: config_file_path,
			mods,
		})
	}

	pub fn save(&self) -> Result<(), failure::Error> {
		let config_file_path_displayable = self.path.display();
		let mut config_file =
			std::fs::File::create(&self.path)
			.with_context(|_| format!("Could not create config file {}", config_file_path_displayable))?;

		let stored_config = StoredConfig::V1 { mods: std::borrow::Cow::Borrowed(&self.mods) };
		serde_json::to_writer_pretty(&mut config_file, &stored_config)
		.with_context(|_| format!("Could not write to config file {}", config_file_path_displayable))?;

		Ok(())
	}
}

fn serialize_config_mods<S>(
	value: &std::collections::HashMap<factorio_mods_common::ModName, factorio_mods_common::ModVersionReq>,
	serializer: S,
) -> Result<S::Ok, S::Error> where S: serde::Serializer {
	let mut map = serializer.serialize_map(None)?;
	for (name, req) in itertools::Itertools::sorted_by(value.iter(), |&(n1, _), &(n2, _)| n1.cmp(n2)) {
		serde::ser::SerializeMap::serialize_key(&mut map, name)?;
		serde::ser::SerializeMap::serialize_value(&mut map, req)?;
	}
	serde::ser::SerializeMap::end(map)
}
