//! API to interface with the local Factorio installation.

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::similar_names,
	clippy::missing_errors_doc,
	clippy::module_name_repetitions,
	clippy::must_use_candidate,
)]

mod api;
pub use self::api::{ API };

mod error;
pub use self::error::{ Error, ErrorKind };

/// A type alias for [`std::result::Result`]
pub type Result<T> = std::result::Result<T, crate::Error>;

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
