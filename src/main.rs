extern crate hyper;

#[macro_use]
mod util;

mod types;
mod api;

fn main() {
	let api = api::API::new(None, None, None);

	let iter = api.search("bobingabout", vec![], None, None, None).unwrap();
	for mod_ in iter {
		match mod_ {
			Ok(mod_) => {
				println!("{:?}", mod_);
			},
			Err(err) => {
				println!("{:?}", err);
				panic!(err)
			}
		}
	}
}
