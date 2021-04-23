//! API to interface with the local Factorio installation.

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::let_unit_value,
	clippy::missing_errors_doc,
	clippy::missing_panics_doc,
	clippy::module_name_repetitions,
	clippy::must_use_candidate,
	clippy::similar_names,
)]

mod api;
pub use self::api::{ Api };

mod error;
pub use self::error::Error;

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
