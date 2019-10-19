//! Common types and functionality used by the other factorio-mods crates.

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::indexing_slicing,
	clippy::must_use_candidate,
	clippy::use_self,
)]

mod types;
pub use self::types::*;

mod util;
pub use self::util::deserialize_string_or_seq_string;
