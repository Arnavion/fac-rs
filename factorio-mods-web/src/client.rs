#![allow(
	clippy::single_match_else,
)]

/// Wraps a `reqwest::unstable::r#async::Client` to only allow limited operations on it.
#[derive(Debug)]
pub(crate) struct Client {
	inner: std::sync::Arc<reqwest::Client>,
}

impl Client {
	/// Creates a new `Client` object.
	pub(crate) fn new(builder: Option<reqwest::ClientBuilder>) -> crate::Result<Self> {
		let builder = builder.unwrap_or_else(reqwest::ClientBuilder::new);

		let mut default_headers = reqwest::header::HeaderMap::new();
		default_headers.insert(
			reqwest::header::USER_AGENT,
			reqwest::header::HeaderValue::from_static(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))));
		let builder = builder.default_headers(default_headers);

		let inner =
			builder
			.redirect(reqwest::redirect::Policy::custom(|attempt| {
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

		Ok(Client { inner: std::sync::Arc::new(inner) })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub(crate) fn get_object<T>(&self, url: reqwest::Url) -> GetObjectFuture<T> where T: serde::de::DeserializeOwned + 'static {
		let inner = self.inner.clone();

		let builder =
			inner.get(url.clone())
			.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone());

		async move {
			let (response, url) = send(builder, url, false).await?;
			Ok(json(response, url).await?)
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn get_zip(&self, url: reqwest::Url, range: Option<&str>) -> GetZipFuture {
		let inner = self.inner.clone();

		let (builder, is_range_request) = {
			let builder = inner.get(url.clone());

			let (builder, is_range_request) =
				if let Some(range) = range {
					(builder.header(reqwest::header::RANGE, range), true)
				}
				else {
					(builder, false)
				};

			(builder.header(reqwest::header::ACCEPT, APPLICATION_ZIP.clone()), is_range_request)
		};

		async move {
			let (response, url) = send(builder, url, is_range_request).await?;
			let url = expect_content_type(&response, url, &APPLICATION_ZIP)?;
			Ok((response, url))
		}
	}

	/// HEADs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn head_zip(&self, url: reqwest::Url) -> HeadZipFuture {
		let inner = self.inner.clone();

		let builder =
			inner.head(url.clone())
			.header(reqwest::header::ACCEPT, APPLICATION_ZIP.clone());

		async move {
			let (response, url) = send(builder, url, false).await?;
			let url = expect_content_type(&response, url, &APPLICATION_ZIP)?;
			Ok((response, url))
		}
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub(crate) fn post_object<B, T>(&self, url: reqwest::Url, body: &B) -> PostObjectFuture<T>
		where B: serde::Serialize, T: serde::de::DeserializeOwned + 'static
	{
		// Separate inner fn so that the impl-trait type alias is independent of B
		async fn post_object_inner<T>(
			inner: std::sync::Arc<reqwest::Client>,
			body: Result<String, serde_urlencoded::ser::Error>,
			url: reqwest::Url,
		) -> crate::Result<(T, reqwest::Url)> where T: serde::de::DeserializeOwned + 'static {
			let body = match body {
				Ok(body) => body,
				Err(err) => return Err(crate::ErrorKind::Serialize(url, err).into()),
			};

			let builder =
				inner.post(url.clone())
				.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone())
				.header(reqwest::header::CONTENT_TYPE, WWW_FORM_URL_ENCODED.clone())
				.body(body.clone());

			let (response, url) = send(builder, url, false).await?;
			Ok(json(response, url).await?)
		}

		let inner = self.inner.clone();
		let body = serde_urlencoded::to_string(body);
		post_object_inner(inner, body, url)
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

pub(crate) type GetObjectFuture<T> = impl std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;
pub(crate) type GetZipFuture = impl std::future::Future<Output = crate::Result<(reqwest::Response, reqwest::Url)>> + 'static;
pub(crate) type HeadZipFuture = impl std::future::Future<Output = crate::Result<(reqwest::Response, reqwest::Url)>> + 'static;
pub(crate) type PostObjectFuture<T> = impl std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;

/// A login failure response.
#[derive(Debug, serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn send(
	builder: reqwest::RequestBuilder,
	url: reqwest::Url,
	is_range_request: bool,
) -> crate::Result<(reqwest::Response, reqwest::Url)> {
	match url.host_str() {
		Some(host) if WHITELISTED_HOSTS.contains(host) => (),
		_ => return Err(crate::ErrorKind::NotWhitelistedHost(url).into()),
	};

	let response = match builder.send().await {
		Ok(response) => response,
		Err(err) => return Err(crate::ErrorKind::Http(url, err).into()),
	};

	match response.status() {
		reqwest::StatusCode::OK if !is_range_request => Ok((response, url)),
		reqwest::StatusCode::PARTIAL_CONTENT if is_range_request => Ok((response, url)),

		reqwest::StatusCode::FOUND => Err(crate::ErrorKind::NotWhitelistedHost(url).into()),

		reqwest::StatusCode::UNAUTHORIZED => {
			let (object, _): (LoginFailureResponse, _) = json(response, url).await?;
			Err(crate::ErrorKind::LoginFailure(object.message).into())
		},

		code => Err(crate::ErrorKind::StatusCode(url, code).into()),
	}
}

async fn json<T>(response: reqwest::Response, url: reqwest::Url) -> crate::Result<(T, reqwest::Url)>
	where T: serde::de::DeserializeOwned + 'static
{
	let url = expect_content_type(&response, url, &APPLICATION_JSON)?;
	match response.json().await {
		Ok(object) => Ok((object, url)),
		Err(err) => Err(crate::ErrorKind::Http(url, err).into()),
	}
}

fn expect_content_type(
	response: &reqwest::Response,
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
