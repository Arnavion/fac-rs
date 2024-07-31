//! Common types and functionality used by the factorio-mods-* crates.

#![deny(missing_docs)]

mod types;
pub use self::types::*;

mod util;
pub use self::util::deserialize_string_or_seq_string;
