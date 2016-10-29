error_chain! {
	foreign_links {
		::hyper::Error, Hyper;
		::serde_json::Error, JSON;
		::std::io::Error, IO;
		::url::ParseError, Parse;
	}

	errors {
		StatusCode(status_code: ::hyper::status::StatusCode) { }
		LoginFailure(message: String) { }
		MalformedModDownloadResponse(message: String) { }
	}
}
