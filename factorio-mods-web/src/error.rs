// Error type for errors returned by this crate.
error_chain! {
	foreign_links {
		// An error from `hyper`
		::hyper::Error, Hyper;

		// Deserializing some JSON failed
		::serde_json::Error, JSON;

		// An error encountered while performing local IO
		::std::io::Error, IO;

		// Parsing a URL failed
		::url::ParseError, Parse;
	}

	errors {
		// An HTTP request did not have a successful status code
		StatusCode(status_code: ::hyper::status::StatusCode) { }

		// A request to the web API resulted in a login failure response
		LoginFailure(message: String) { }

		// A mod release had a malformed filename
		MalformedModReleaseFilename(path: ::std::path::PathBuf) { }

		// Trying to download a mod from the mods portal returned a malformed response
		MalformedModDownloadResponse(message: String) { }
	}
}
