/// Errors returned by this crate.
#[derive(Debug)]
pub enum Error {
	/// Could not deserialize HTTP response.
	Deserialize(url::Url, serde_json::Error),

	/// Could not perform HTTP request.
	Http(url::Url, hyper::Error),

	/// Specified HTTP range is malformed.
	InvalidRange(String, http::header::InvalidHeaderValue),

	/// A request to the web API resulted in a login failure response.
	LoginFailure(String),

	/// Got a malformed HTTP response.
	MalformedResponse(url::Url, String),

	/// Tried to request a host that isn't whitelisted.
	NotWhitelistedHost(url::Url),

	/// Parsing a URL failed.
	Parse(String, url::ParseError),

	/// Parsing a URL failed.
	ParseUri(url::Url, http::uri::InvalidUri),

	/// Could not serialize HTTP POST request body.
	Serialize(url::Url, serde_urlencoded::ser::Error),

	/// An HTTP request did not have a successful status code.
	StatusCode(url::Url, http::StatusCode),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Deserialize(url, _) => write!(f, "could not deserialize response body for URL {}", url),
			Error::Http(url, _) => write!(f, "could not fetch URL {}", url),
			Error::InvalidRange(range, _) => write!(f, "could not parse HTTP range {}", range),
			Error::LoginFailure(message) => write!(f, "login failed: {}", message),
			Error::MalformedResponse(url, message) => write!(f, "request to URL {} got malformed response: {}", url, message),
			Error::NotWhitelistedHost(url) => write!(f, "host of {} is not whitelisted", url),
			Error::Parse(url, _) => write!(f, "could not parse URL {}", url),
			Error::ParseUri(url, _) => write!(f, "could not parse URL {}", url),
			Error::Serialize(url, _) => write!(f, "could not serialize request body for URL {}", url),
			Error::StatusCode(url, status_code) => write!(f, "request to URL {} returned {}", url, status_code),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		#[allow(clippy::match_same_arms)]
		match self {
			Error::Deserialize(_, err) => Some(err),
			Error::Http(_, err) => Some(err),
			Error::InvalidRange(_, err) => Some(err),
			Error::LoginFailure(_) => None,
			Error::MalformedResponse(_, _) => None,
			Error::NotWhitelistedHost(_) => None,
			Error::Parse(_, err) => Some(err),
			Error::ParseUri(_, err) => Some(err),
			Error::Serialize(_, err) => Some(err),
			Error::StatusCode(_, _) => None,
		}
	}
}
