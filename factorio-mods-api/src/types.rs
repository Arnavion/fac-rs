extern crate serde;
extern crate serde_json;

make_deserializable!(pub struct ModId(u64));

make_deserializable!(pub struct DownloadCount(u64));

make_deserializable!(pub struct Mod {
	id: ModId,

	owner: String,

	name: String,
	title: String,
	summary: String,

	downloads_count: DownloadCount,
});
