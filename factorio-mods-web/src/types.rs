/// A date and time string.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::newtype_display,
	derive_struct::newtype_fromstr,
	serde_derive::Deserialize,
)]
pub struct DateTime(pub String);

/// Number of downloads.
#[derive(
	Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::newtype_display,
	serde_derive::Deserialize,
)]
pub struct DownloadCount(pub u64);

/// A mod object returned by `API::get`.
#[derive(Clone, Debug, PartialEq, serde_derive::Deserialize)]
pub struct Mod {
	/// The name of the mod.
	pub name: factorio_mods_common::ModName,

	/// The title of the mod.
	pub title: factorio_mods_common::ModTitle,

	/// The authors of the mod.
	#[serde(deserialize_with = "factorio_mods_common::deserialize_string_or_seq_string")]
	pub owner: Vec<factorio_mods_common::AuthorName>,

	/// A short summary of the mod.
	pub summary: ModSummary,

	/// All the releases of the mod.
	pub releases: Vec<ModRelease>,

	/// The number of times the mod has been downloaded.
	pub downloads_count: DownloadCount,
}

/// The summary of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::newtype_display,
	derive_struct::newtype_fromstr,
	serde_derive::Deserialize,
)]
pub struct ModSummary(pub String);

/// A single mod release.
#[derive(Clone, Debug, PartialEq, serde_derive::Deserialize)]
pub struct ModRelease {
	/// The version of the mod release.
	pub version: factorio_mods_common::ReleaseVersion,

	/// The `info.json` of the mod release.
	pub info_json: ModReleaseInfo,

	/// The URL to download the mod release.
	pub download_url: factorio_mods_common::Url,

	/// The filename of the mod release.
	#[serde(rename(deserialize = "file_name"))]
	pub filename: Filename,

	/// The date and time at which the mod release was created.
	pub released_at: DateTime,

	/// The hash of the mod release file.
	pub sha1: ModHash,
}

/// Extra information about a single mod release.
#[derive(Clone, Debug, PartialEq, serde_derive::Deserialize)]
pub struct ModReleaseInfo {
	/// The versions of the game supported by the mod release.
	pub factorio_version: factorio_mods_common::ModVersionReq,
}

/// The hash of a mod release file.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::newtype_display,
	derive_struct::newtype_fromstr,
	serde_derive::Deserialize,
)]
pub struct ModHash(pub String);

/// The filename of a mod release.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	derive_struct::newtype_display,
	derive_struct::newtype_fromstr,
	serde_derive::Deserialize,
)]
pub struct Filename(pub String);

/// A mod object returned by `API::search`.
#[derive(Clone, Debug, PartialEq, serde_derive::Deserialize)]
pub struct SearchResponseMod {
	/// The name of the mod.
	pub name: factorio_mods_common::ModName,

	/// The title of the mod.
	pub title: factorio_mods_common::ModTitle,

	/// The authors of the mod.
	#[serde(deserialize_with = "factorio_mods_common::deserialize_string_or_seq_string")]
	pub owner: Vec<factorio_mods_common::AuthorName>,

	/// A short summary of the mod.
	pub summary: ModSummary,

	/// The latest release of the mod.
	pub latest_release: ModRelease,

	/// The number of times the mod has been downloaded.
	pub downloads_count: DownloadCount,
}
