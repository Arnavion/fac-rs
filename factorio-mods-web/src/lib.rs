//! API to interface with <https://mods.factorio.com/>

#![feature(
	async_await,
	type_alias_impl_trait,
)]

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::large_enum_variant,
	clippy::module_name_repetitions,
	clippy::use_self,
)]

pub use reqwest;

mod api;
pub use self::api::{ API, DownloadResponse, GetResponse, LoginResponse, SearchResponse };

mod client;

mod error;
pub use self::error::{ Error, ErrorKind };

/// A type alias for [`std::result::Result`]
pub type Result<T> = std::result::Result<T, crate::Error>;

mod types;
pub use self::types::*;
