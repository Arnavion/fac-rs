make_struct!(pub DateTime(String));

make_struct!(pub RatingCount(u64));

make_struct!(pub DownloadCount(u64));

make_struct!(pub VisitCount(u64));

make_struct!(pub GameVersion(::semver::VersionReq));

make_struct!(pub LicenseName(String));

make_struct!(pub LicenseFlags(u64));

make_struct!(pub Url(String));

make_struct!(pub Mod {
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
});

make_struct!(pub ModId(u64));

make_struct!(pub ModName(String));

make_struct!(pub AuthorNames(Vec<String>));
impl ::std::fmt::Display for AuthorNames {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", self.0.join(", "))
	}
}

make_struct!(pub ModTitle(String));

make_struct!(pub ModSummary(String));

make_struct!(pub ModDescription(String));

make_struct!(pub ModRelease {
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
});

make_struct!(pub ReleaseId(u64));

make_struct!(pub ReleaseVersion(::semver::Version));

make_struct!(pub Filename(String));

make_struct!(pub FileSize(u64));

make_struct!(pub ReleaseInfo {
	author: AuthorNames,
	description: Option<ModDescription>,
	factorio_version: GameVersion,
	homepage: Option<Url>,
	name: ModName,
	title: ModTitle,
	version: ReleaseVersion,
});

make_struct!(pub Tag {
	id: TagId,
	name: TagName,
	title: TagTitle,
	description: TagDescription,
	#[serde(rename(deserialize = "type"))]
	type_name: TagType,
});

make_struct!(pub Tags(Vec<Tag>));
impl ::std::fmt::Display for Tags {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", ::itertools::join(self.0.iter().map(|t| &t.name), ", "))
	}
}

make_struct!(pub TagId(u64));

make_struct!(pub TagName(String));

make_struct!(pub TagTitle(String));

make_struct!(pub TagDescription(String));

make_struct!(pub TagType(String));

make_struct!(pub UserCredentials {
	username: ServiceUsername,
	token: ServiceToken,
});

make_struct!(pub ServiceUsername(String));

make_struct!(pub ServiceToken(String));


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
