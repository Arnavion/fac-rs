extern crate serde;
extern crate serde_json;

make_deserializable!(pub struct DateTime(String));

make_deserializable!(pub struct DownloadCount(u64));

make_deserializable!(pub struct GameVersion(String));

make_deserializable!(pub struct Url(String));

make_deserializable!(pub struct Mod {
	id: ModId,

	name: ModName,
	owner: AuthorNames,
	title: ModTitle,
	summary: ModDescription,

	github_path: Url,
	homepage: Url,

	game_versions: Vec<GameVersion>,

	created_at: DateTime,
	latest_release: ModRelease,

	current_user_rating: Option<serde_json::Value>,
	downloads_count: DownloadCount,
	tags: Vec<Tag>,
});

make_deserializable!(pub struct ModId(u64));

make_deserializable!(pub struct ModName(String));

make_deserializable!(pub struct AuthorNames(Vec<String>));

make_deserializable!(pub struct ModTitle(String));

make_deserializable!(pub struct ModDescription(String));

make_deserializable!(pub struct ModRelease {
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

make_deserializable!(pub struct ReleaseId(u64));

make_deserializable!(pub struct ReleaseVersion(String));

make_deserializable!(pub struct Filename(String));

make_deserializable!(pub struct FileSize(u64));

make_deserializable!(pub struct ReleaseInfo {
	author: AuthorNames,
	/* description: ModDescription, # Can't represent since `description` isn't present in every ReleaseInfo */
	factorio_version: GameVersion,
	/* homepage: Url, # Can't represent since `homepage` isn't present in every ReleaseInfo */
	name: ModName,
	title: ModTitle,
	version: ReleaseVersion,
});

make_deserializable!(pub struct Tag {
	id: TagId,
	name: TagName,
	title: TagTitle,
	description: TagDescription,
	/* type: TagType, # Can't represent since `type` is a keyword */
});

make_deserializable!(pub struct TagId(u64));

make_deserializable!(pub struct TagName(String));

make_deserializable!(pub struct TagTitle(String));

make_deserializable!(pub struct TagDescription(String));

make_deserializable!(pub struct TagType(String));
