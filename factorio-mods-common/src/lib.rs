#![deny(missing_docs)]
#![feature(proc_macro)]

//! Common types and functionality used by the other factorio-mods crates.

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate derive_struct;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod types;
pub use types::*;
