/// A URL.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct Url(pub String);

/// The name of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct ModName(pub String);

impl serde::Serialize for ModName {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		serializer.serialize_str(&self.0)
	}
}

/// The name of an author of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct AuthorName(pub String);

/// The title of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct ModTitle(pub String);

/// The description of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct ModDescription(pub String);

/// The version of a mod release.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	serde::Deserialize,
)]
pub struct ReleaseVersion(#[serde(deserialize_with = "deserialize_version")] pub semver::Version);

/// A username and token used with the parts of the web API that require authentication.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct UserCredentials {
	/// The username.
	pub username: ServiceUsername,

	/// The token.
	pub token: ServiceToken,
}

/// A username used with the parts of the web API that require authentication.
#[derive(
	Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct ServiceUsername(pub String);

/// A token used with the parts of the web API that require authentication.
#[derive(
	Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::NewTypeDisplay,
	derive_struct::NewTypeFromStr,
	serde::Deserialize,
)]
pub struct ServiceToken(pub String);

/// The specification of a dependency in a mod's info.json
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dependency {
	/// The name of the dependency.
	pub name: ModName,

	/// The version of the dependency.
	pub version: ModVersionReq,

	/// The kind of the dependency.
	pub kind: package::DependencyKind,
}

impl<'de> serde::Deserialize<'de> for Dependency {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
		struct Visitor;

		impl serde::de::Visitor<'_> for Visitor {
			type Value = Dependency;

			fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				formatter.write_str("a string")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
				parse_dependency(v)
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}

impl<'a> package::Dependency<'a, ReleaseVersion> for Dependency {
	type Name = ModName;
	type VersionReq = VersionReqMatcher<'a>;

	fn name(&self) -> &Self::Name {
		&self.name
	}

	fn version_req(&'a self) -> Self::VersionReq {
		VersionReqMatcher {
			version_req: &self.version.0,
			is_base: self.name.0 == "base",
		}
	}

	fn kind(&self) -> package::DependencyKind {
		self.kind
	}
}

/// A version requirement.
#[derive(
	Clone, Debug, Eq, PartialEq,
	derive_struct::NewTypeDisplay,
	serde::Deserialize,
)]
pub struct ModVersionReq(#[serde(deserialize_with = "deserialize_version_req")] pub semver::VersionReq);

impl serde::Serialize for ModVersionReq {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		serializer.serialize_str(&self.0.to_string())
	}
}

/// A version requirement matcher used by the package solver.
#[derive(Debug)]
pub struct VersionReqMatcher<'a> {
	/// The version requirement.
	pub version_req: &'a semver::VersionReq,

	/// Whether this version requirement is for the base mod or not.
	pub is_base: bool,
}

impl package::VersionReq<ReleaseVersion> for VersionReqMatcher<'_> {
	fn matches(&self, other: &ReleaseVersion) -> bool {
		if self.version_req.matches(&other.0) {
			return true;
		}

		if self.is_base && other.0.major == 1 && other.0.minor == 0 {
			let other = semver::Version { major: 0, minor: 18, patch: 99, pre: semver::Prerelease::EMPTY, build: semver::BuildMetadata::EMPTY };
			return self.version_req.matches(&other);
		}

		false
	}
}

/// Deserializes a `semver::Version`
fn deserialize_version<'de, D>(deserializer: D) -> Result<semver::Version, D::Error> where D: serde::Deserializer<'de> {
	struct Visitor;

	impl serde::de::Visitor<'_> for Visitor {
		type Value = semver::Version;

		fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			formatter.write_str("semver::Version")
		}

		fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E> where E: serde::de::Error {
			v.parse().or_else(|_| fixup_version(v).parse()).map_err(serde::de::Error::custom)
		}
	}

	deserializer.deserialize_str(Visitor)
}

/// Deserializes a `semver::VersionReq`
fn deserialize_version_req<'de, D>(deserializer: D) -> Result<semver::VersionReq, D::Error> where D: serde::Deserializer<'de> {
	struct Visitor;

	impl serde::de::Visitor<'_> for Visitor {
		type Value = semver::VersionReq;

		fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			formatter.write_str("semver::VersionReq")
		}

		fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E> where E: serde::de::Error {
			parse_version_req(v).map_err(serde::de::Error::custom)
		}
	}

	deserializer.deserialize_str(Visitor)
}

/// Parses a `semver::VersionReq` from a string.
fn parse_version_req(s: &str) -> Result<semver::VersionReq, semver::Error> {
	s.parse().or_else(|_| fixup_version(s).parse())
}

/// Fixes up some bad version strings returned by the web API into something valid for the `semver` crate.
pub fn fixup_version(s: &str) -> String {
	itertools::join(s.split('.').map(|part|
		if part.starts_with('0') {
			let rest = part.trim_start_matches('0');
			if rest.is_empty() {
				"0"
			}
			else {
				rest
			}
		}
		else {
			part
		}), ".")
}

/// Parses the given string as a Dependency
fn parse_dependency<E>(s: &str) -> Result<Dependency, E> where E: serde::de::Error {
	static DEPENDENCY_REGEX: once_cell::sync::Lazy<regex::Regex> =
		once_cell::sync::Lazy::new(|| regex::Regex::new(r"^((?:\?|\(\?\)|!|~)?)\s*([^<>=]+?)\s*((<|<=|=|>=|>)\s*([\d\.]+))?\s*$").unwrap());

	let captures = DEPENDENCY_REGEX.captures(s)
		.ok_or_else(|| serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &"a valid dependency specifier"))?;

	let kind = match &captures[1] {
		"!" => package::DependencyKind::Conflicts,
		"?" | "(?)" => package::DependencyKind::Optional,
		"" | "~" => package::DependencyKind::Required,
		_ => unreachable!(),
	};

	let name = ModName(captures[2].to_owned());

	let version_req_string = captures.get(3).map_or("*", |m| m.as_str());
	let version_req =
		parse_version_req(version_req_string)
		.or_else(|_| {
			let fixed_version = format!("{}{}", &captures[4], fixup_version(&captures[5]));
			fixed_version.parse()
				.map_err(|err| serde::de::Error::custom(format!("invalid dependency specifier {fixed_version:?}: {err}")))
		})?;

	Ok(Dependency { name, version: ModVersionReq(version_req), kind, })
}

#[cfg(test)]
mod tests {
	fn test_deserialize_release_version_inner(s: &str, expected: &str) {
		let expected = super::ReleaseVersion(expected.parse().unwrap());
		let actual: super::ReleaseVersion = serde_json::from_str(s).unwrap();
		assert_eq!(actual, expected);
	}

	#[test]
	fn test_deserialize_release_version() {
		test_deserialize_release_version_inner(r#""0.2.2""#, "0.2.2");
		test_deserialize_release_version_inner(r#""0.14.0""#, "0.14.0");
		test_deserialize_release_version_inner(r#""0.2.02""#, "0.2.2");
		test_deserialize_release_version_inner(r#""0.14.00""#, "0.14.0");
		test_deserialize_release_version_inner(r#""016.0.5""#, "16.0.5");
	}

	fn test_deserialize_dependency_inner(s: &str, name: &str, version: &str, kind: package::DependencyKind) {
		let expected = super::Dependency { name: super::ModName(name.to_owned()), version: super::ModVersionReq(version.parse().unwrap()), kind };
		let actual: super::Dependency = serde_json::from_str(s).unwrap();
		assert_eq!(actual, expected);
	}

	#[test]
	fn test_deserialize_dependency() {
		test_deserialize_dependency_inner(r#""base""#, "base", "*", package::DependencyKind::Required);
		test_deserialize_dependency_inner(r#""? base""#, "base", "*", package::DependencyKind::Optional);
		test_deserialize_dependency_inner(r#""?base""#, "base", "*", package::DependencyKind::Optional);
		test_deserialize_dependency_inner(r#""(?)base""#, "base", "*", package::DependencyKind::Optional);
		test_deserialize_dependency_inner(r#""base >= 0.14.0""#, "base", ">=0.14.0", package::DependencyKind::Required);
		test_deserialize_dependency_inner(r#""? base >= 0.14.0""#, "base", ">=0.14.0", package::DependencyKind::Optional);
		test_deserialize_dependency_inner(r#""base >= 0.14.00""#, "base", ">=0.14.0", package::DependencyKind::Required);
		test_deserialize_dependency_inner(r#""some name with spaces >= 1.2.3""#, "some name with spaces", ">=1.2.3", package::DependencyKind::Required);
		test_deserialize_dependency_inner(r#""? some name with spaces >= 1.2.3""#, "some name with spaces", ">=1.2.3", package::DependencyKind::Optional);
		test_deserialize_dependency_inner(r#""!foo""#, "foo", "*", package::DependencyKind::Conflicts);
	}
}
