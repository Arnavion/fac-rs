make_newtype!(pub DateTime(String));

make_newtype!(pub RatingCount(u64));

make_newtype!(pub DownloadCount(u64));

make_newtype!(pub VisitCount(u64));

make_newtype!(pub GameVersion(::semver::VersionReq));

make_newtype!(pub LicenseName(String));

make_newtype!(pub LicenseFlags(u64));

make_newtype!(pub Url(String));

make_deserializable!(pub struct Mod {
	pub id: ModId,

	pub name: ModName,
	pub owner: AuthorNames,
	pub title: ModTitle,
	pub summary: ModSummary,
	pub description: ModDescription,

	pub github_path: Url,
	pub homepage: Url,
	pub license_name: LicenseName,
	pub license_url: Url,
	pub license_flags: LicenseFlags,

	pub game_versions: Vec<GameVersion>,

	pub created_at: DateTime,
	pub updated_at: DateTime,
	pub releases: Vec<ModRelease>,

	pub ratings_count: RatingCount,
	pub current_user_rating: Option<::serde_json::Value>,
	pub downloads_count: DownloadCount,
	pub visits_count: VisitCount,
	pub tags: Tags,
});

make_newtype!(pub ModId(u64));

make_newtype!(pub ModName(String));

make_deserializable!(pub struct AuthorNames(pub Vec<String>));
impl ::std::fmt::Display for AuthorNames {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", self.0.join(", "))
	}
}

make_newtype!(pub ModTitle(String));

make_newtype!(pub ModSummary(String));

make_newtype!(pub ModDescription(String));

make_deserializable!(pub struct ModRelease {
	pub id: ReleaseId,
	pub version: ReleaseVersion,
	pub factorio_version: GameVersion,
	pub game_version: GameVersion,

	pub download_url: Url,
	pub file_name: Filename,
	pub file_size: FileSize,
	pub released_at: DateTime,

	pub downloads_count: DownloadCount,

	pub info_json: ReleaseInfo,
});

make_newtype!(pub ReleaseId(u64));

make_newtype!(pub ReleaseVersion(::semver::Version));

make_newtype!(pub Filename(String));

make_newtype!(pub FileSize(u64));

make_deserializable!(pub struct ReleaseInfo {
	pub author: AuthorNames,
	pub description: Option<ModDescription>,
	pub factorio_version: GameVersion,
	pub homepage: Option<Url>,
	pub name: ModName,
	pub title: ModTitle,
	pub version: ReleaseVersion,
});

make_deserializable!(pub struct Tag {
	pub id: TagId,
	pub name: TagName,
	pub title: TagTitle,
	pub description: TagDescription,
	pub type_name: TagType,
});

make_deserializable!(pub struct Tags(pub Vec<Tag>));
impl ::std::fmt::Display for Tags {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", ::itertools::join(self.0.iter().map(|t| &t.name), ", "))
	}
}

make_newtype!(pub TagId(u64));

make_newtype!(pub TagName(String));

make_newtype!(pub TagTitle(String));

make_newtype!(pub TagDescription(String));

make_newtype!(pub TagType(String));

make_deserializable!(pub struct UserCredentials {
	pub username: ServiceUsername,
	pub token: ServiceToken,
});

make_newtype!(pub ServiceUsername(String));

make_newtype!(pub ServiceToken(String));


pub fn fixup_version(s: &str) -> String {
	::itertools::join(s.split('.').enumerate().map(|(i, part)| {
		let part =
			if i == 0 && part.len() >= 1 && part.chars().next().unwrap() == '0' {
				"0".to_string() + part.trim_matches('0')
			}
			else {
				part.trim_matches('0').to_string()
			};

		let part = if part.is_empty() { "0".to_string() } else { part };

		part
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
