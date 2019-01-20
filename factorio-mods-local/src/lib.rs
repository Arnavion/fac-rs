//! API to interface with the local Factorio installation.

#![feature(
	generators,
	generator_trait,
	unrestricted_attribute_tokens,
)]

#![deny(missing_docs)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::default_trait_access,
	clippy::similar_names,
	clippy::module_name_repetitions,
	clippy::too_many_arguments,
	clippy::use_self,
)]

#[macro_use] extern crate lazy_static;

mod api;
pub use self::api::{ API };

mod error;
pub use self::error::{ Error, ErrorKind };

/// A type alias for [`std::result::Result`]
pub type Result<T> = std::result::Result<T, crate::Error>;

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
