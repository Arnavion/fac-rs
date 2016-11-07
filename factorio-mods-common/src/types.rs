#[derive(newtype)]
pub struct DateTime(String);

#[derive(newtype)]
pub struct RatingCount(u64);

#[derive(newtype)]
pub struct DownloadCount(u64);

#[derive(newtype)]
pub struct VisitCount(u64);

#[derive(newtype)]
pub struct GameVersion(::semver::VersionReq);

#[derive(newtype)]
pub struct LicenseName(String);

#[derive(newtype)]
pub struct LicenseFlags(u64);

#[derive(newtype)]
pub struct Url(String);

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Mod {
	id: ModId,

	name: ModName,
	owner: AuthorNames,
	title: ModTitle,
	summary: ModSummary,
	description: ModDescription,

	github_path: Url,
	homepage: Url,
	license_name: LicenseName,
	license_url: Url,
	license_flags: LicenseFlags,

	game_versions: Vec<GameVersion>,

	created_at: DateTime,
	updated_at: DateTime,
	releases: Vec<ModRelease>,

	ratings_count: RatingCount,
	// current_user_rating: ???, # Unknown type
	downloads_count: DownloadCount,
	visits_count: VisitCount,
	tags: Tags,
}

#[derive(newtype)]
pub struct ModId(u64);

#[derive(newtype)]
pub struct ModName(String);

#[derive(newtype)]
pub struct AuthorNames(Vec<String>);
impl ::std::fmt::Display for AuthorNames {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", self.0.join(", "))
	}
}

#[derive(newtype)]
pub struct ModTitle(String);

#[derive(newtype)]
pub struct ModSummary(String);

#[derive(newtype)]
pub struct ModDescription(String);

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ModRelease {
	id: ReleaseId,
	version: ReleaseVersion,
	factorio_version: GameVersion,
	game_version: GameVersion,

	download_url: Url,
	file_name: Filename,
	file_size: FileSize,
	released_at: DateTime,

	downloads_count: DownloadCount,

	info_json: ReleaseInfo,
}

#[derive(newtype)]
pub struct ReleaseId(u64);

#[derive(newtype)]
pub struct ReleaseVersion(::semver::Version);

#[derive(newtype)]
pub struct Filename(String);

#[derive(newtype)]
pub struct FileSize(u64);

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ReleaseInfo {
	author: AuthorNames,
	description: Option<ModDescription>,
	factorio_version: GameVersion,
	homepage: Option<Url>,
	name: ModName,
	title: ModTitle,
	version: ReleaseVersion,
}

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Tag {
	id: TagId,
	name: TagName,
	title: TagTitle,
	description: TagDescription,
	#[serde(rename(deserialize = "type"))]
	type_name: TagType,
}

#[derive(newtype)]
pub struct Tags(Vec<Tag>);
impl ::std::fmt::Display for Tags {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", ::itertools::join(self.0.iter().map(|t| &t.name), ", "))
	}
}

#[derive(newtype)]
pub struct TagId(u64);

#[derive(newtype)]
pub struct TagName(String);

#[derive(newtype)]
pub struct TagTitle(String);

#[derive(newtype)]
pub struct TagDescription(String);

#[derive(newtype)]
pub struct TagType(String);

#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct UserCredentials {
	username: ServiceUsername,
	token: ServiceToken,
}

#[derive(newtype)]
pub struct ServiceUsername(String);

#[derive(newtype)]
pub struct ServiceToken(String);


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
