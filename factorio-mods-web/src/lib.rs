//! API to interface with <https://mods.factorio.com/>

#![deny(missing_docs)]
#![feature(catch_expr, conservative_impl_trait, generators, generator_trait, proc_macro)]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(
	large_enum_variant,
	missing_docs_in_private_items,
	result_unwrap_used,
	shadow_reuse,
	stutter,
	too_many_arguments,
	unseparated_literal_suffix,
	use_debug,
	use_self,
))]

#[macro_use]
extern crate error_chain;
extern crate derive_error_chain;
extern crate derive_new;
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
pub use api::{ API, SearchOrder };

mod client;

mod error;
pub use error::{ Error, ErrorKind, Result };

mod search;
pub use search::{ PageNumber, ResponseNumber, SearchResponseMod };

mod types;
pub use types::*;
