#![feature(proc_macro)]

extern crate appdirs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate factorio_mods_common;
extern crate glob;
#[macro_use]
extern crate lazy_static;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate zip;

mod api;
pub use api::{ API };

mod error;
pub use error::{ Error, ErrorKind, Result, };

mod installed_mod;
pub use installed_mod::{ InstalledMod, InstalledModIterator, };
