error_chain! {
	foreign_links {
		::hyper::Error, Hyper;
		::serde_json::Error, JSON;
		::url::ParseError, Parse;
	}

	errors {
		StatusCode(status_code: ::hyper::status::StatusCode) { }
	}
}
