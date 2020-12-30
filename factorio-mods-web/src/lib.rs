//! API to search mods / download mods / show mod info from <https://mods.factorio.com/>

#![deny(missing_docs, rust_2018_idioms, warnings)]

#![deny(clippy::all, clippy::pedantic)]
#![allow(
	clippy::default_trait_access,
	clippy::missing_errors_doc,
	clippy::similar_names,
	clippy::type_complexity,
)]

mod api;
pub use self::api::API;

mod client;

mod error;
pub use self::error::Error;

mod types;
pub use self::types::*;
