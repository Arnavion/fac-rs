#[derive(Debug)]
pub enum InstalledMod {
	Zipped,
	Unpacked,
}

impl InstalledMod {
	pub fn find(name: Option<::factorio_mods_common::ModName>, version: Option<::factorio_mods_common::ReleaseVersion>) {
	}
}

#[derive(Debug)]
pub struct InstalledModIterator;

impl Iterator for InstalledModIterator {
	type Item = InstalledMod;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}
