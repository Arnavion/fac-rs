#![feature(proc_macro)]

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate factorio_mods_common;
extern crate hyper;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

mod api;
pub use api::{ API, SearchResultsIterator };

mod error;
pub use error::{ Error, ErrorKind, Result };
