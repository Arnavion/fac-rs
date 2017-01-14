/// A date and time string.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct DateTime(String);

/// Number of ratings.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct RatingCount(u64);

/// Number of downloads.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct DownloadCount(u64);

/// Number of visits.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct VisitCount(u64);

/// A mod object returned by `API::get`.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Mod {
	/// The mod ID.
	id: ModId,

	/// The name of the mod.
	name: ::factorio_mods_common::ModName,

	/// The authors of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	owner: Vec<::factorio_mods_common::AuthorName>,

	/// The title of the mod.
	title: ::factorio_mods_common::ModTitle,

	/// A short summary of the mod.
	summary: ModSummary,

	/// A longer description of the mod.
	description: ::factorio_mods_common::ModDescription,

	/// The URL of the GitHub repository of the mod.
	github_path: ::factorio_mods_common::Url,

	/// The URL of the homepage of the mod.
	homepage: ::factorio_mods_common::Url,

	/// The name of the mod's license.
	license_name: LicenseName,

	/// The URL of the mod's license.
	license_url: ::factorio_mods_common::Url,

	/// The flags of the mod's license.
	license_flags: LicenseFlags,

	/// The versions of the game supported by the mod.
	game_versions: Vec<::factorio_mods_common::ModVersionReq>,

	/// The date and time at which the mod was created.
	created_at: DateTime,

	/// The date and time at which the mod was last updated.
	updated_at: DateTime,

	/// All the releases of the mod.
	releases: Vec<ModRelease>,

	/// The number of user ratings the mod has received.
	ratings_count: RatingCount,

	// current_user_rating: ???, # Unknown type

	/// The number of times the mod has been downloaded.
	downloads_count: DownloadCount,

	/// The number of times the mod page has been visited.
	visits_count: VisitCount,

	/// The tags of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	tags: Vec<Tag>,
}

/// A mod ID.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModId(u64);

/// The summary of a mod.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ModSummary(String);

/// The name of a mod's license.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct LicenseName(String);

/// License flags.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct LicenseFlags(u64);

/// A single mod release.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct ModRelease {
	/// The ID of the mod release.
	id: ReleaseId,

	/// The version of the mod release.
	version: ::factorio_mods_common::ReleaseVersion,

	/// The versions of the game supported by the mod release.
	factorio_version: ::factorio_mods_common::ModVersionReq,

	/// The URL to download the mod release.
	download_url: ::factorio_mods_common::Url,

	/// The filename of the mod release.
	#[serde(rename(deserialize = "file_name"))]
	filename: Filename,

	/// The file size of the mod release.
	file_size: FileSize,

	/// The date and time at which the mod release was created.
	released_at: DateTime,

	/// The number of times the mod release has been downloaded.
	downloads_count: DownloadCount,

	/// The `info.json` of the mod release.
	info_json: ::factorio_mods_common::ModInfo,
}

/// The ID of a mod release.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct ReleaseId(u64);

/// The filename of a mod release.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct Filename(String);

/// The file size of a mod release.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct FileSize(u64);

/// A tag.
#[derive(Clone, Debug, Deserialize, new, getters)]
pub struct Tag {
	/// The ID of the tag.
	id: TagId,

	/// The name of the tag.
	name: TagName,

	/// The title of the tag.
	title: TagTitle,

	/// The description of the tag.
	description: TagDescription,

	/// The type of the tag.
	#[serde(rename(deserialize = "type"))]
	type_name: TagType,
}

/// The ID of a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct TagId(u64);

/// The name of a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct TagName(String);

/// The title of a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct TagTitle(String);

/// The description of a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct TagDescription(String);

/// The type of a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, new, newtype_display, newtype_ref)]
pub struct TagType(String);
