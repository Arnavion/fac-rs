/// A URL.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct Url(String);

/// The name of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModName(String);

impl ::serde::Serialize for ModName {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
		serializer.serialize_str(&self.0)
	}
}

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
	factorio_version: ModVersionReq,

	/// The URL of the homepage of the mod release.
	homepage: Option<Url>,

	/// Dependencies
	#[serde(default = "default_dependencies")]
	#[serde(deserialize_with = "::deserialize_string_or_seq_string")]
	dependencies: Vec<Dependency>,
}

/// The specification of a dependency in a mod's info.json
#[derive(Clone, Debug, PartialEq, new, getters)]
pub struct Dependency {
	/// The name of the dependency.
	name: ModName,

	/// The version of the dependency.
	version: ModVersionReq,

	/// Whether the dependency is required or not.
	required: bool,
}

impl<'de> ::serde::Deserialize<'de> for Dependency {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: ::serde::Deserializer<'de> {
		struct Visitor;

		impl<'de> ::serde::de::Visitor<'de> for Visitor {
			type Value = Dependency;

			fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				formatter.write_str("a string")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: ::serde::de::Error {
				parse_dependency(v)
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}

/// A version requirement.
#[derive(Clone, Debug, PartialEq, new, newtype_deserialize, newtype_display, newtype_ref)]
pub struct ModVersionReq(::semver::VersionReq);

impl ::serde::Serialize for ModVersionReq {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::Serializer {
		serializer.serialize_str(&self.0.to_string())
	}
}

lazy_static! {
	static ref DEFAULT_GAME_VERSION: ModVersionReq = ModVersionReq::new("0.12".parse().unwrap());
}

/// Generates a copy of the default game version.
///
/// Used as the default value of the `factorio_version` field in a mod's `info.json` if the field doesn't exist.
fn default_game_version() -> ModVersionReq {
	DEFAULT_GAME_VERSION.clone()
}

/// Parses the given string as a `::semver::Version`
fn parse_version(s: &str) -> Result<::semver::Version, ::semver::SemVerError> {
	s.parse().or_else(|_| fixup_version(s).parse())
}

/// Parses the given string as a `::semver::VersionReq`
fn parse_version_req(s: &str) -> Result<::semver::VersionReq, ::semver::ReqParseError> {
	s.parse().or_else(|_| fixup_version(s).parse())
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

/// The default dependencies of a mod.
///
/// Used as the default value of the `dependencies` field in a mod's `info.json` if the field doesn't exist.
fn default_dependencies() -> Vec<Dependency> {
	DEFAULT_DEPENDENCIES.clone()
}

lazy_static! {
	static ref DEPENDENCY_REGEX: ::regex::Regex = ::regex::Regex::new(r"^(\??)\s*([^<>=]+?)\s*((<|<=|=|>=|>)\s*([\d\.]+))?\s*$").unwrap();
	static ref DEFAULT_DEPENDENCIES: Vec<Dependency> = vec![Dependency {
		name: ModName("base".to_string()),
		version: ModVersionReq(::semver::VersionReq::any()),
		required: true,
	}];
}

/// Parses the given string as a Dependency
fn parse_dependency<E>(s: &str) -> Result<Dependency, E> where E: ::serde::de::Error {
	let captures = DEPENDENCY_REGEX.captures(s)
		.ok_or_else(|| ::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(s), &"a valid dependency specifier"))?;

	let required = captures[1].is_empty();

	let name = ModName::new(captures[2].to_string());

	let version_req_string = captures.get(3).map_or("*", |m| m.as_str());
	let version_req =
		parse_version_req(version_req_string)
		.or_else(|_| {
			let fixed_version = captures[4].to_string() + &fixup_version(&captures[5]);
			fixed_version.parse()
				.map_err(|err| ::serde::de::Error::custom(format!("invalid dependency specifier {:?}: {}", &fixed_version, ::std::error::Error::description(&err))))
		})?;

	Ok(Dependency { name: ModName(name.to_string()), version: ModVersionReq(version_req), required, })
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_deserialize_release_version_inner(s: &str, expected: &str) {
		let expected = ReleaseVersion(expected.parse().unwrap());
		let actual: ReleaseVersion = ::serde_json::from_str(s).unwrap();
		assert_eq!(actual, expected);
	}

	#[test]
	fn test_deserialize_release_version() {
		test_deserialize_release_version_inner(r#""0.2.2""#, "0.2.2");
		test_deserialize_release_version_inner(r#""0.14.0""#, "0.14.0");
		test_deserialize_release_version_inner(r#""0.2.02""#, "0.2.2");
		test_deserialize_release_version_inner(r#""0.14.00""#, "0.14.0");
	}

	fn test_deserialize_dependency_inner(s: &str, name: &str, version: &str, required: bool) {
		let expected = Dependency { name: ModName(name.to_string()), version: ModVersionReq(version.parse().unwrap()), required };
		let actual: Dependency = ::serde_json::from_str(s).unwrap();
		assert_eq!(actual, expected);
	}

	#[test]
	fn test_deserialize_dependency() {
		test_deserialize_dependency_inner(r#""base""#, "base", "*", true);
		test_deserialize_dependency_inner(r#""? base""#, "base", "*", false);
		test_deserialize_dependency_inner(r#""?base""#, "base", "*", false);
		test_deserialize_dependency_inner(r#""base >= 0.14.0""#, "base", ">=0.14.0", true);
		test_deserialize_dependency_inner(r#""? base >= 0.14.0""#, "base", ">=0.14.0", false);
		test_deserialize_dependency_inner(r#""base >= 0.14.00""#, "base", ">=0.14.0", true);
		test_deserialize_dependency_inner(r#""some name with spaces >= 1.2.3""#, "some name with spaces", ">=1.2.3", true);
		test_deserialize_dependency_inner(r#""? some name with spaces >= 1.2.3""#, "some name with spaces", ">=1.2.3", false);
	}
}
