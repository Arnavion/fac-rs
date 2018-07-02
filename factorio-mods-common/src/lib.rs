//! Common types and functionality used by the other factorio-mods crates.

#![deny(missing_docs)]
#![feature(proc_macro, proc_macro_path_invoc)]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(
	const_static_lifetime,
	indexing_slicing,
	use_self,
))]

extern crate derive_struct;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "package")]
extern crate package;
extern crate regex;
extern crate semver;
#[macro_use]
extern crate serde;
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

mod types;
pub use types::*;

mod util;
pub use util::deserialize_string_or_seq_string;
