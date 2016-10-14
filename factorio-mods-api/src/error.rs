error_chain! {
	foreign_links {
		::hyper::Error, Hyper;
		::url::ParseError, Parse;
		::serde_json::Error, JSON;
	}

	errors {
		StatusCode(status_code: ::hyper::status::StatusCode) { }
	}
}
