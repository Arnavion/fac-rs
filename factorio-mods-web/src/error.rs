/// Error kinds for errors returned by this crate.
#[derive(Debug, error_chain)]
pub enum ErrorKind {
	/// A generic error message
	Msg(String),

	/// An error from `reqwest`
	#[error_chain(foreign)]
	Reqwest(::reqwest::Error),

	/// Parsing a URL failed
	#[error_chain(foreign)]
	Parse(::reqwest::UrlError),

	/// An HTTP request did not have a successful status code
	#[error_chain(custom)]
	StatusCode(::reqwest::StatusCode),

	/// A request to the web API resulted in a login failure response
	#[error_chain(custom)]
	LoginFailure(String),

	/// Got a malformed HTTP response
	#[error_chain(custom)]
	MalformedResponse(String),
}
