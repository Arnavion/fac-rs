#![deny(missing_docs)]

//! Common types and functionality used by the other factorio-mods crates.

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
