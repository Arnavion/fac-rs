//! API to interface with <https://mods.factorio.com/>

#![deny(missing_docs)]
#![feature(catch_expr, generators, generator_trait, proc_macro_non_items, proc_macro_path_invoc, unrestricted_attribute_tokens)]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(
	const_static_lifetime,
	stutter,
	use_self,
))]

extern crate derive_error_chain;
#[macro_use]
extern crate error_chain;
extern crate derive_struct;
extern crate factorio_mods_common;
extern crate futures_await as futures;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
pub extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_urlencoded;
pub extern crate tokio_core;

use futures::prelude::{ async_block, async_stream_block, await, stream_yield };

mod api;
pub use api::{ API };

mod client;

mod error;
pub use error::{ Error, ErrorKind, Result };

mod types;
pub use types::*;
