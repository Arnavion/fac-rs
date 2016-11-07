/// A date and time string.
#[derive(newtype)]
pub struct DateTime(String);

/// Number of ratings.
#[derive(newtype)]
pub struct RatingCount(u64);

/// Number of downloads.
#[derive(newtype)]
pub struct DownloadCount(u64);

/// Number of visits.
#[derive(newtype)]
pub struct VisitCount(u64);

/// A mod object returned by `API::get`.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Mod {
	id: ModId,

	name: ::factorio_mods_common::ModName,
	owner: ::factorio_mods_common::AuthorNames,
	title: ::factorio_mods_common::ModTitle,
	summary: ModSummary,
	description: ::factorio_mods_common::ModDescription,

	github_path: ::factorio_mods_common::Url,
	homepage: ::factorio_mods_common::Url,
	license_name: LicenseName,
	license_url: ::factorio_mods_common::Url,
	license_flags: LicenseFlags,

	game_versions: Vec<::factorio_mods_common::GameVersion>,

	created_at: DateTime,
	updated_at: DateTime,
	releases: Vec<ModRelease>,

	ratings_count: RatingCount,
	// current_user_rating: ???, # Unknown type
	downloads_count: DownloadCount,
	visits_count: VisitCount,
	tags: Tags,
}

/// A mod ID.
#[derive(newtype)]
pub struct ModId(u64);

/// The summary of a mod.
#[derive(newtype)]
pub struct ModSummary(String);

/// The name of the license.
#[derive(newtype)]
pub struct LicenseName(String);

/// License flags.
#[derive(newtype)]
pub struct LicenseFlags(u64);

/// A single mod release.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ModRelease {
	id: ReleaseId,
	version: ::factorio_mods_common::ReleaseVersion,
	factorio_version: ::factorio_mods_common::GameVersion,
	game_version: ::factorio_mods_common::GameVersion,

	download_url: ::factorio_mods_common::Url,
	file_name: Filename,
	file_size: FileSize,
	released_at: DateTime,

	downloads_count: DownloadCount,

	info_json: ReleaseInfo,
}

/// The ID of a mod release.
#[derive(newtype)]
pub struct ReleaseId(u64);

/// The filename of a mod release.
#[derive(newtype)]
pub struct Filename(String);

/// The file size of a mod release.
#[derive(newtype)]
pub struct FileSize(u64);

/// Detailed information of a mod release.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ReleaseInfo {
	author: ::factorio_mods_common::AuthorNames,
	description: Option<::factorio_mods_common::ModDescription>,
	factorio_version: ::factorio_mods_common::GameVersion,
	homepage: Option<::factorio_mods_common::Url>,
	name: ::factorio_mods_common::ModName,
	title: ::factorio_mods_common::ModTitle,
	version: ::factorio_mods_common::ReleaseVersion,
}

/// A tag.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Tag {
	id: TagId,
	name: TagName,
	title: TagTitle,
	description: TagDescription,
	#[serde(rename(deserialize = "type"))]
	type_name: TagType,
}

/// A collection of tags.
#[derive(newtype)]
pub struct Tags(Vec<Tag>);
impl ::std::fmt::Display for Tags {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", ::itertools::join(self.0.iter().map(|t| &t.name), ", "))
	}
}

/// The ID of a tag.
#[derive(newtype)]
pub struct TagId(u64);

/// The name of a tag.
#[derive(newtype)]
pub struct TagName(String);

/// The title of a tag.
#[derive(newtype)]
pub struct TagTitle(String);

/// The description of a tag.
#[derive(newtype)]
pub struct TagDescription(String);

/// The type of a tag.
#[derive(newtype)]
pub struct TagType(String);
