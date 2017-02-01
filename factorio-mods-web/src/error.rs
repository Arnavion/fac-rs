/// Error kinds for errors returned by this crate.
#[derive(Debug, error_chain)]
pub enum ErrorKind {
	/// A generic error message
	Msg(String),

	/// Could not perform HTTP request
	#[error_chain(custom)]
	#[error_chain(display = r#"|url, _| write!(f, "Could not fetch URL {}", url)"#)]
	#[error_chain(cause = "|_, err| err")]
	HTTP(::reqwest::Url, ::reqwest::Error),

	/// Parsing a URL failed
	#[error_chain(custom)]
	#[error_chain(display = r#"|url, _| write!(f, "Could not parse URL {}", url)"#)]
	#[error_chain(cause = "|_, err| err")]
	Parse(String, ::reqwest::UrlError),

	/// An HTTP request did not have a successful status code
	#[error_chain(custom)]
	#[error_chain(display = r#"|url, code| write!(f, "Request to URL {} returned {}", url, code)"#)]
	StatusCode(::reqwest::Url, ::reqwest::StatusCode),

	/// A request to the web API resulted in a login failure response
	#[error_chain(custom)]
	#[error_chain(display = r#"|message| write!(f, "Login failed: {}", message)"#)]
	LoginFailure(String),

	/// Got a malformed HTTP response
	#[error_chain(custom)]
	#[error_chain(display = r#"|url, reason| write!(f, "Request to URL {} was malformed: {}", url, reason)"#)]
	MalformedResponse(::reqwest::Url, String),

	/// Received a redirect to a host that isn't in the allowed list
	#[error_chain(custom)]
	#[error_chain(display = r#"|url| write!(f, "Unexpected redirect to {}", url)"#)]
	UnexpectedRedirect(::reqwest::Url),
}
