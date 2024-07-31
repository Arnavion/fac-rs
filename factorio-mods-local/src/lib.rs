//! API to interface with the local Factorio installation.

#![deny(missing_docs)]

mod api;
pub use self::api::{ Api };

mod error;
pub use self::error::Error;

mod installed_mod;
pub use self::installed_mod::{ InstalledMod, InstalledModType, ModInfo };
