//! API to interface with the local Factorio installation.

#![deny(missing_docs)]
#![feature(generators, generator_trait, proc_macro_path_invoc, use_extern_macros)]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(
	const_static_lifetime,
	similar_names,
	stutter,
	too_many_arguments,
	use_self,
))]

extern crate appdirs;
extern crate derive_error_chain;
#[macro_use]
extern crate error_chain;
extern crate derive_struct;
extern crate factorio_mods_common;
extern crate globset;
#[macro_use]
extern crate lazy_static;
extern crate semver;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate zip;

mod api;
pub use api::{ API };

mod error;
pub use error::{ Error, ErrorKind, Result, };

mod installed_mod;
pub use installed_mod::{ InstalledMod, InstalledModType, ModInfo };
