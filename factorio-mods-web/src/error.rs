/// Errors returned by this crate.
#[derive(Debug)]
pub struct Error {
	kind: ErrorKind,
	backtrace: failure::Backtrace,
}

impl Error {
	/// Gets the kind of error
	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}
}

impl failure::Fail for Error {
	fn cause(&self) -> Option<&dyn failure::Fail> {
		self.kind.cause()
	}

	fn backtrace(&self) -> Option<&failure::Backtrace> {
		Some(&self.backtrace)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.kind.fmt(f)
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Self {
		Error {
			kind,
			backtrace: Default::default(),
		}
	}
}

/// Error kinds for errors returned by this crate.
#[derive(Debug, failure_derive::Fail)]
pub enum ErrorKind {
	/// Could not create HTTP client
	#[fail(display = "Could not create HTTP client")]
	CreateClient(#[cause] reqwest::Error),

	/// Could not perform HTTP request
	#[fail(display = "Could not fetch URL {}", _0)]
	HTTP(reqwest::Url, #[cause] reqwest::Error),

	/// Parsing a URL failed
	#[fail(display = "Could not parse URL {}", _0)]
	Parse(String, #[cause] reqwest::UrlError),

	/// An HTTP request did not have a successful status code
	#[fail(display = "Request to URL {} returned {}", _0, _1)]
	StatusCode(reqwest::Url, reqwest::StatusCode),

	/// A request to the web API resulted in a login failure response
	#[fail(display = "Login failed: {}", _0)]
	LoginFailure(String),

	/// Got a malformed HTTP response
	#[fail(display = "Request to URL {} got malformed response: {}", _0, _1)]
	MalformedResponse(reqwest::Url, String),

	/// Tried to request a host that isn't whitelisted
	#[fail(display = "Host {} is not whitelisted", _0)]
	NotWhitelistedHost(reqwest::Url),

	/// Could not serialize HTTP POST request body
	#[fail(display = "Could not serialize request body for URL {}", _0)]
	Serialize(reqwest::Url, #[cause] serde_urlencoded::ser::Error),
}
