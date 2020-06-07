//! API to interface with <https://mods.factorio.com/>

#![feature(
	type_alias_impl_trait,
)]

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::missing_errors_doc,
	clippy::module_name_repetitions,
)]

#![recursion_limit = "256"]

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
