//! Common types and functionality used by the other factorio-mods crates.

#![deny(missing_docs)]
#![feature(proc_macro, proc_macro_path_invoc)]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(
	missing_docs_in_private_items,
	option_unwrap_used,
	result_unwrap_used,
	string_add,
	too_many_arguments,
	unseparated_literal_suffix,
	use_debug,
))]

extern crate derive_new;
extern crate derive_struct;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
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
