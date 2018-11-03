//! API to interface with <https://mods.factorio.com/>

#![feature(
	arbitrary_self_types,
	async_await,
	await_macro,
	futures_api,
	pin,
	unrestricted_attribute_tokens,
)]

#![deny(missing_docs)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::const_static_lifetime,
	clippy::large_enum_variant,
	clippy::stutter,
	clippy::use_self,
)]

#[macro_use] extern crate lazy_static;
pub extern crate reqwest;

mod api;
pub use self::api::{ API };

mod client;

mod error;
pub use self::error::{ Error, ErrorKind, Result };

mod types;
pub use self::types::*;
