#![allow(
	clippy::single_match_else,
)]

/// Wraps a `reqwest::unstable::r#async::Client` to only allow limited operations on it.
#[derive(Debug)]
pub struct Client {
	inner: reqwest::r#async::Client,
}

impl Client {
	/// Creates a new `Client` object.
	pub fn new(builder: Option<reqwest::r#async::ClientBuilder>) -> crate::Result<Self> {
		let builder = builder.unwrap_or_else(reqwest::r#async::ClientBuilder::new);

		let mut default_headers = reqwest::header::HeaderMap::new();
		default_headers.insert(
			reqwest::header::USER_AGENT,
			reqwest::header::HeaderValue::from_static(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))));
		let builder = builder.default_headers(default_headers);

		let inner =
			builder
			.redirect(reqwest::RedirectPolicy::custom(|attempt| {
				if match attempt.url().host_str() {
					Some(host) if WHITELISTED_HOSTS.contains(host) => true,
					_ => false,
				} {
					attempt.follow()
				}
				else {
					attempt.stop()
				}
			}))
			.build()
			.map_err(crate::ErrorKind::CreateClient)?;

		Ok(Client { inner })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub fn get_object<T>(&self, url: reqwest::Url) -> GetObjectFuture<T> where T: serde::de::DeserializeOwned + 'static {
		let builder = self.inner.get(url.clone());

		async {
			let builder = builder.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone());
			let (response, url) = await!(send(builder, url))?;
			Ok(await!(json(response, url))?)
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip(&self, url: reqwest::Url) -> impl std::future::Future<Output = crate::Result<(reqwest::r#async::Response, reqwest::Url)>> + 'static {
		let builder = self.inner.get(url.clone());

		async {
			let builder = builder.header(reqwest::header::ACCEPT, APPLICATION_ZIP.clone());
			let (response, url) = await!(send(builder, url))?;
			let url = expect_content_type(&response, url, &APPLICATION_ZIP)?;
			Ok((response, url))
		}
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<B, T>(&self, url: reqwest::Url, body: &B) -> PostObjectFuture<T>
		where B: serde::Serialize, T: serde::de::DeserializeOwned + 'static
	{
		async fn inner<T>(
			url: reqwest::Url,
			builder: reqwest::r#async::RequestBuilder,
			body: Result<String, serde_urlencoded::ser::Error>,
		) -> crate::Result<(T, reqwest::Url)> where T: serde::de::DeserializeOwned + 'static {
			let body = match body {
				Ok(body) => body,
				Err(err) => return Err(crate::ErrorKind::Serialize(url, err).into()),
			};

			let builder =
				builder
				.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone())
				.header(reqwest::header::CONTENT_TYPE, WWW_FORM_URL_ENCODED.clone())
				.body(body);

			let (response, url) = await!(send(builder, url))?;
			Ok(await!(json(response, url))?)
		}

		let builder = self.inner.post(url.clone());

		let body = serde_urlencoded::to_string(body);

		inner(url, builder, body)
	}
}

lazy_static::lazy_static! {
	static ref WHITELISTED_HOSTS: std::collections::HashSet<&'static str> = vec![
		"auth.factorio.com",
		"direct.mods-data.factorio.com",
		"mods.factorio.com",
		"mods-data.factorio.com",
	].into_iter().collect();
	static ref APPLICATION_JSON: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/json");
	static ref APPLICATION_ZIP: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/zip");
	static ref WWW_FORM_URL_ENCODED: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded");
}

pub(crate) existential type GetObjectFuture<T>: std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;
pub(crate) existential type PostObjectFuture<T>: std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;

/// A login failure response.
#[derive(Debug, serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn send(
	builder: reqwest::r#async::RequestBuilder,
	url: reqwest::Url,
) -> crate::Result<(reqwest::r#async::Response, reqwest::Url)> {
	match url.host_str() {
		Some(host) if WHITELISTED_HOSTS.contains(host) => (),
		_ => return Err(crate::ErrorKind::NotWhitelistedHost(url).into()),
	};

	let response = match await!(futures_util::compat::Future01CompatExt::compat(builder.send())) {
		Ok(response) => response,
		Err(err) => return Err(crate::ErrorKind::HTTP(url, err).into()),
	};

	match response.status() {
		reqwest::StatusCode::OK => Ok((response, url)),

		reqwest::StatusCode::UNAUTHORIZED => {
			let (object, _): (LoginFailureResponse, _) = await!(json(response, url))?;
			Err(crate::ErrorKind::LoginFailure(object.message).into())
		},

		reqwest::StatusCode::FOUND => Err(crate::ErrorKind::NotWhitelistedHost(url).into()),

		code => Err(crate::ErrorKind::StatusCode(url, code).into()),
	}
}

async fn json<T>(mut response: reqwest::r#async::Response, url: reqwest::Url) -> crate::Result<(T, reqwest::Url)>
	where T: serde::de::DeserializeOwned + 'static
{
	let url = expect_content_type(&response, url, &APPLICATION_JSON)?;
	match await!(futures_util::compat::Future01CompatExt::compat(response.json())) {
		Ok(object) => Ok((object, url)),
		Err(err) => Err(crate::ErrorKind::HTTP(url, err).into()),
	}
}

fn expect_content_type(
	response: &reqwest::r#async::Response,
	url: reqwest::Url,
	expected_mime: &reqwest::header::HeaderValue,
) -> crate::Result<reqwest::Url> {
	let mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
		Some(mime) => mime,
		None => return Err(crate::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string()).into()),
	};

	if mime != expected_mime {
		return Err(crate::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {:?}", mime)).into());
	}

	Ok(url)
}
