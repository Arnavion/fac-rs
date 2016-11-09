#![deny(missing_docs)]
#![feature(conservative_impl_trait, proc_macro)]

//! API to interface with https://mods.factorio.com/

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate error_chain;
extern crate factorio_mods_common;
extern crate hyper;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_struct;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

mod api;
pub use api::{ API, SearchOrder };

mod error;
pub use error::{ Error, ErrorKind, Result };

mod search;
pub use search::{ PageNumber, ResponseNumber, SearchResponseMod };

mod types;
pub use types::*;

mod util;
