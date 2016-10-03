extern crate serde;
extern crate serde_json;

make_deserializable!(pub struct ModId(u64));

make_deserializable!(pub struct Mod {
	title: String,
	name: String,
	id: ModId,
	summary: String,
	owner: String,
});
