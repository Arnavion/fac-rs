#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate factorio_mods_common;
extern crate hyper;
extern crate itertools;
extern crate serde;
extern crate serde_json;
extern crate url;

mod api;
pub use api::{ API, SearchResultsIterator };

mod error;
pub use error::{ Error, ErrorKind, Result };
