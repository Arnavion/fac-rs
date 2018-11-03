//! Common types and functionality used by the other factorio-mods crates.

#![deny(missing_docs)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::indexing_slicing,
	clippy::use_self,
)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;

#[cfg(test)]
extern crate serde_json;

mod types;
pub use self::types::*;

mod util;
pub use self::util::deserialize_string_or_seq_string;
