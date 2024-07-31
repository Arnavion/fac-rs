//! API to search mods / download mods / show mod info from <https://mods.factorio.com/>

#![deny(missing_docs)]

mod api;
pub use self::api::Api;

mod client;

mod error;
pub use self::error::Error;

mod types;
pub use self::types::*;
