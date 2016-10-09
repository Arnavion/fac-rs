use installed_mod;

#[derive(Debug)]
pub struct Config;

#[derive(Debug)]
pub struct Manager {
	config: Config,
}

impl Manager {
	pub fn new() -> Manager {
		Manager {
			config: Config,
		}
	}

	pub fn installed_mods(&self) -> installed_mod::InstalledModIterator {
		installed_mod::InstalledModIterator
	}
}
