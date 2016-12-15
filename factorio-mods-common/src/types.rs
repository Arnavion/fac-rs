/// The required game version.
#[derive(Clone, Debug, PartialEq, new, newtype_deserialize, newtype_display, newtype_ref)]
pub struct GameVersion(::semver::VersionReq);

/// A URL.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct Url(String);

/// The name of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModName(String);

/// The name of an author of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct AuthorName(String);

/// The title of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModTitle(String);

/// The description of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModDescription(String);

/// The version of a mod release.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_deserialize, newtype_display, newtype_ref)]
pub struct ReleaseVersion(::semver::Version);

/// A username and token used with the parts of the web API that require authentication.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct UserCredentials {
	/// The username.
	username: ServiceUsername,

	/// The token.
	token: ServiceToken,
}

/// A username used with the parts of the web API that require authentication.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ServiceUsername(String);

/// A token used with the parts of the web API that require authentication.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ServiceToken(String);

/// Represents the contents of `info.json` of a mod release.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ModInfo {
	/// The name of the mod release.
	name: ModName,

	/// The authors of the mod release.
	#[serde(deserialize_with = "::deserialize_string_or_seq_string")]
	author: Vec<AuthorName>,

	/// The title of the mod release.
	title: ModTitle,

	/// A longer description of the mod release.
	description: Option<ModDescription>,

	/// The version of the mod release.
	version: ReleaseVersion,

	/// The versions of the game supported by the mod release.
	#[serde(default = "default_game_version")]
	factorio_version: GameVersion,

	/// The URL of the homepage of the mod release.
	homepage: Option<Url>,
}

lazy_static! {
	static ref DEFAULT_GAME_VERSION: GameVersion = GameVersion::new(::semver::VersionReq::parse("0.12").unwrap());
}

/// Generates a copy of the default game version.
///
/// Used as the default value of the `factorio_version` field in a mod's `info.json` if the field doesn't exist.
fn default_game_version() -> GameVersion {
	DEFAULT_GAME_VERSION.clone()
}

/// Parses the given string as a ::semver::Version
fn parse_version(s: &str) -> Result<::semver::Version, ::semver::SemVerError> {
	if let Ok(version) = ::semver::Version::parse(s) {
		Ok(version)
	}
	else {
		let fixed_version = fixup_version(s);
		::semver::Version::parse(&fixed_version)
	}
}

/// Parses the given string as a ::semver::VersionReq
fn parse_version_req(s: &str) -> Result<::semver::VersionReq, ::semver::ReqParseError> {
	if let Ok(version_req) = ::semver::VersionReq::parse(s) {
		Ok(version_req)
	}
	else {
		let fixed_version = fixup_version(s);
		::semver::VersionReq::parse(&fixed_version)
	}
}

/// Fixes up some bad version strings returned by the web API into something valid for the `semver` crate.
pub fn fixup_version(s: &str) -> String {
	::itertools::join(s.split('.').enumerate().map(|(i, part)| {
		let part =
			if i == 0 && part.len() >= 1 && part.chars().next().unwrap() == '0' {
				"0".to_string() + part.trim_matches('0')
			}
			else {
				part.trim_matches('0').to_string()
			};

		if part.is_empty() {
			"0".to_string()
		}
		else {
			part
		}
	}), ".")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn good_one() {
		let result = fixup_version("0.2.2");
		println!("{}", result);
		assert!(result == "0.2.2");
	}

	#[test]
	fn good_two() {
		let result = fixup_version("0.14.0");
		println!("{}", result);
		assert!(result == "0.14.0");
	}

	#[test]
	fn bad_one() {
		let result = fixup_version("0.2.02");
		println!("{}", result);
		assert!(result == "0.2.2");
	}

	#[test]
	fn bad_two() {
		let result = fixup_version("0.14.00");
		println!("{}", result);
		assert!(result == "0.14.0");
	}
}
