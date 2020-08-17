/// Errors returned by this crate.
#[derive(Debug)]
pub enum Error {
	/// Could not create HTTP client.
	CreateClient(reqwest::Error),

	/// Could not perform HTTP request.
	Http(reqwest::Url, reqwest::Error),

	/// A request to the web API resulted in a login failure response.
	LoginFailure(String),

	/// Got a malformed HTTP response.
	MalformedResponse(reqwest::Url, String),

	/// Tried to request a host that isn't whitelisted.
	NotWhitelistedHost(reqwest::Url),

	/// Parsing a URL failed.
	Parse(String, url::ParseError),

	/// Could not serialize HTTP POST request body.
	Serialize(reqwest::Url, serde_urlencoded::ser::Error),

	/// An HTTP request did not have a successful status code.
	StatusCode(reqwest::Url, reqwest::StatusCode),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::CreateClient(_) => f.write_str("could not create HTTP client"),
			Error::Http(url, _) => write!(f, "could not fetch URL {}", url),
			Error::LoginFailure(message) => write!(f, "login failed: {}", message),
			Error::MalformedResponse(url, message) => write!(f, "request to URL {} got malformed response: {}", url, message),
			Error::NotWhitelistedHost(url) => write!(f, "host {} is not whitelisted", url),
			Error::Parse(url, _) => write!(f, "could not parse URL {}", url),
			Error::Serialize(url, _) => write!(f, "could not serialize request body for URL {}", url),
			Error::StatusCode(url, status_code) => write!(f, "request to URL {} returned {}", url, status_code),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		#[allow(clippy::match_same_arms)]
		match self {
			Error::CreateClient(err) => Some(err),
			Error::Http(_, err) => Some(err),
			Error::LoginFailure(_) => None,
			Error::MalformedResponse(_, _) => None,
			Error::NotWhitelistedHost(_) => None,
			Error::Parse(_, err) => Some(err),
			Error::Serialize(_, err) => Some(err),
			Error::StatusCode(_, _) => None,
		}
	}
}
