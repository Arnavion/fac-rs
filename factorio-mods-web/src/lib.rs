#![deny(missing_docs)]
#![feature(conservative_impl_trait)]

//! API to interface with https://mods.factorio.com/

extern crate backtrace;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate derive_error_chain;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate derive_struct;
extern crate factorio_mods_common;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
pub extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod api;
pub use api::{ API, SearchOrder };

mod client;

mod error;
pub use error::{ Error, ErrorKind, Result };

mod search;
pub use search::{ PageNumber, ResponseNumber, SearchResponseMod };

mod types;
pub use types::*;
