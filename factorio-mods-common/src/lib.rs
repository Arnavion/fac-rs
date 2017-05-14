//! Common types and functionality used by the other factorio-mods crates.

#![deny(missing_docs)]

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

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate derive_struct;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate semver;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

mod types;
pub use types::*;

mod util;
pub use util::deserialize_string_or_seq_string;
