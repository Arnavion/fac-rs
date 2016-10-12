extern crate appdirs;
extern crate backtrace;
#[macro_use]
extern crate factorio_mods_common;
extern crate glob;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate zip;

mod installed_mod;

mod manager;
pub use manager::*;

mod types;
