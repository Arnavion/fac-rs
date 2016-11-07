/// The required game version.
#[derive(newtype)]
pub struct GameVersion(::semver::VersionReq);

/// A URL.
#[derive(newtype)]
pub struct Url(String);

/// The name of a mod.
#[derive(newtype)]
pub struct ModName(String);

/// The names of the authors of a mod.
#[derive(newtype)]
pub struct AuthorNames(Vec<String>);
impl ::std::fmt::Display for AuthorNames {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", self.0.join(", "))
	}
}

/// The title of a mod.
#[derive(newtype)]
pub struct ModTitle(String);

/// The description of a mod.
#[derive(newtype)]
pub struct ModDescription(String);

/// The version of a mod release.
#[derive(newtype)]
pub struct ReleaseVersion(::semver::Version);

/// A username and token used with the parts of the web API that require authentication.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct UserCredentials {
	username: ServiceUsername,
	token: ServiceToken,
}

/// A username used with the parts of the web API that require authentication.
#[derive(newtype)]
pub struct ServiceUsername(String);

/// A token used with the parts of the web API that require authentication.
#[derive(newtype)]
pub struct ServiceToken(String);

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
