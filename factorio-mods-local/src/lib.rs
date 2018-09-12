//! API to interface with the local Factorio installation.

#![feature(
	generators,
	generator_trait,
	tool_lints,
	unrestricted_attribute_tokens,
)]

#![deny(missing_docs)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::similar_names,
	clippy::stutter,
	clippy::too_many_arguments,
	clippy::use_self,
)]

#[macro_use] extern crate lazy_static;

mod api;
pub use self::api::{ API };

mod error;
pub use self::error::{ Error, ErrorKind, Result, };

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
