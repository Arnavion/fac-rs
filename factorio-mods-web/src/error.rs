/// Errors returned by this crate.
#[derive(Debug)]
pub struct Error {
	/// The kind of the error.
	pub kind: ErrorKind,

	/// The backtrace of the error.
	pub backtrace: backtrace::Backtrace,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{}", self.kind)?;
		writeln!(f)?;
		writeln!(f, "{:?}", self.backtrace)?;
		Ok(())
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		self.kind.source()
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
#[derive(Debug)]
pub enum ErrorKind {
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

impl std::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorKind::CreateClient(_) => f.write_str("could not create HTTP client"),
			ErrorKind::Http(url, _) => write!(f, "could not fetch URL {}", url),
			ErrorKind::LoginFailure(message) => write!(f, "login failed: {}", message),
			ErrorKind::MalformedResponse(url, message) => write!(f, "request to URL {} got malformed response: {}", url, message),
			ErrorKind::NotWhitelistedHost(url) => write!(f, "host {} is not whitelisted", url),
			ErrorKind::Parse(url, _) => write!(f, "could not parse URL {}", url),
			ErrorKind::Serialize(url, _) => write!(f, "could not serialize request body for URL {}", url),
			ErrorKind::StatusCode(url, status_code) => write!(f, "request to URL {} returned {}", url, status_code),
		}
	}
}

impl std::error::Error for ErrorKind {
	#[allow(clippy::match_same_arms)]
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			ErrorKind::CreateClient(err) => Some(err),
			ErrorKind::Http(_, err) => Some(err),
			ErrorKind::LoginFailure(_) => None,
			ErrorKind::MalformedResponse(_, _) => None,
			ErrorKind::NotWhitelistedHost(_) => None,
			ErrorKind::Parse(_, err) => Some(err),
			ErrorKind::Serialize(_, err) => Some(err),
			ErrorKind::StatusCode(_, _) => None,
		}
	}
}
