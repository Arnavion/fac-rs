/// A date and time string.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct DateTime(String);

/// Number of downloads.
#[derive(
	Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct DownloadCount(u64);

/// A mod object returned by `API::get`.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters, ::serde_derive::Deserialize)]
pub struct Mod {
	/// The name of the mod.
	name: ::factorio_mods_common::ModName,

	/// The title of the mod.
	title: ::factorio_mods_common::ModTitle,

	/// The authors of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	owner: Vec<::factorio_mods_common::AuthorName>,

	/// A short summary of the mod.
	summary: ModSummary,

	/// All the releases of the mod.
	releases: Vec<ModRelease>,

	/// The number of times the mod has been downloaded.
	#[getter(copy)]
	downloads_count: DownloadCount,
}

/// The summary of a mod.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct ModSummary(String);

/// A single mod release.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters, ::serde_derive::Deserialize)]
pub struct ModRelease {
	/// The version of the mod release.
	version: ::factorio_mods_common::ReleaseVersion,

	/// The `info.json` of the mod release.
	info_json: ModReleaseInfo,

	/// The URL to download the mod release.
	download_url: ::factorio_mods_common::Url,

	/// The filename of the mod release.
	#[serde(rename(deserialize = "file_name"))]
	filename: Filename,

	/// The date and time at which the mod release was created.
	released_at: DateTime,

	/// The hash of the mod release file.
	sha1: ModHash,
}

/// Extra information about a single mod release.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters, ::serde_derive::Deserialize)]
pub struct ModReleaseInfo {
	/// The versions of the game supported by the mod release.
	factorio_version: ::factorio_mods_common::ModVersionReq,
}

/// The hash of a mod release file.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct ModHash(String);

/// The filename of a mod release.
#[derive(
	Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
	::derive_new::new,
	::derive_struct::newtype_display, ::derive_struct::newtype_ref,
	::serde_derive::Deserialize,
)]
pub struct Filename(String);

/// A mod object returned by `API::search`.
#[derive(Clone, Debug, PartialEq, ::derive_new::new, ::derive_struct::getters, ::serde_derive::Deserialize)]
pub struct SearchResponseMod {
	/// The name of the mod.
	name: ::factorio_mods_common::ModName,

	/// The title of the mod.
	title: ::factorio_mods_common::ModTitle,

	/// The authors of the mod.
	#[serde(deserialize_with = "::factorio_mods_common::deserialize_string_or_seq_string")]
	owner: Vec<::factorio_mods_common::AuthorName>,

	/// A short summary of the mod.
	summary: ModSummary,

	/// The latest release of the mod.
	latest_release: ModRelease,

	/// The number of times the mod has been downloaded.
	#[getter(copy)]
	downloads_count: DownloadCount,
}
