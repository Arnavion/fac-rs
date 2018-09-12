/// Error kinds for errors returned by this crate.
#[derive(Debug, derive_error_chain::ErrorChain)]
pub enum ErrorKind {
	/// Could not create HTTP client
	#[error_chain(custom)]
	#[error_chain(display = const("Could not create HTTP client"))]
	#[error_chain(cause = |err| err)]
	CreateClient(reqwest::Error),

	/// Could not perform HTTP request
	#[error_chain(custom)]
	#[error_chain(display = const("Could not fetch URL {0}"))]
	#[error_chain(cause = |_, err| err)]
	HTTP(reqwest::Url, reqwest::Error),

	/// Parsing a URL failed
	#[error_chain(custom)]
	#[error_chain(display = const("Could not parse URL {0}"))]
	#[error_chain(cause = |_, err| err)]
	Parse(String, reqwest::UrlError),

	/// An HTTP request did not have a successful status code
	#[error_chain(custom)]
	#[error_chain(display = const("Request to URL {0} returned {1}"))]
	StatusCode(reqwest::Url, reqwest::StatusCode),

	/// A request to the web API resulted in a login failure response
	#[error_chain(custom)]
	#[error_chain(display = const("Login failed: {0}"))]
	LoginFailure(String),

	/// Got a malformed HTTP response
	#[error_chain(custom)]
	#[error_chain(display = const("Request to URL {0} got malformed response: {1}"))]
	MalformedResponse(reqwest::Url, String),

	/// Tried to request a host that isn't whitelisted
	#[error_chain(custom)]
	#[error_chain(display = const("Host {0} is not whitelisted"))]
	NotWhitelistedHost(reqwest::Url),

	/// Could not serialize HTTP POST request body
	#[error_chain(custom)]
	#[error_chain(display = const("Could not serialize request body for URL {0}"))]
	#[error_chain(cause = |_, err| err)]
	Serialize(reqwest::Url, serde_urlencoded::ser::Error),
}
