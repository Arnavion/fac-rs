#![allow(missing_docs)]

// Error type for errors returned by this crate.
error_chain! {
	foreign_links {
		// An error from `hyper`
		::hyper::Error, Hyper;

		// Deserializing some JSON failed
		::serde_json::Error, JSON;

		// Parsing a URL failed
		::url::ParseError, Parse;
	}

	errors {
		// An HTTP request did not have a successful status code
		StatusCode(status_code: ::hyper::status::StatusCode) { }

		// A request to the web API resulted in a login failure response
		LoginFailure(message: String) { }

		// Trying to download a mod from the mods portal returned a malformed response
		MalformedModDownloadResponse(message: String) { }
	}
}
