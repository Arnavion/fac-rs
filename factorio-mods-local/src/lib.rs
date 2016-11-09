#![deny(missing_docs)]
#![feature(conservative_impl_trait, proc_macro)]

//! API to interface with the local Factorio installation.

extern crate appdirs;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate error_chain;
extern crate factorio_mods_common;
extern crate glob;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_struct;
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
pub use installed_mod::{ InstalledMod, InstalledModType };
