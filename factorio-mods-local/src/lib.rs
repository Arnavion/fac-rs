//! API to interface with the local Factorio installation.

#![feature(
	generators,
	generator_trait,
)]

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::default_trait_access,
	clippy::similar_names,
	clippy::module_name_repetitions,
	clippy::too_many_arguments,
	clippy::use_self,
)]

mod api;
pub use self::api::{ API };

mod error;
pub use self::error::{ Error, ErrorKind };

/// A type alias for [`std::result::Result`]
pub type Result<T> = std::result::Result<T, crate::Error>;

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
