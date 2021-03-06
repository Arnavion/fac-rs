//! Common types and functionality used by the factorio-mods-* crates.

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::missing_errors_doc,
	clippy::must_use_candidate,
)]

mod types;
pub use self::types::*;

mod util;
pub use self::util::deserialize_string_or_seq_string;
