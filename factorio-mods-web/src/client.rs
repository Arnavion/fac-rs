#![allow(
	clippy::single_match_else,
)]

/// Wraps a `reqwest::unstable::r#async::Client` to only allow limited operations on it.
#[derive(Debug)]
pub(crate) struct Client {
	inner: std::sync::Arc<reqwest::r#async::Client>,
}

impl Client {
	/// Creates a new `Client` object.
	pub(crate) fn new(builder: Option<reqwest::r#async::ClientBuilder>) -> crate::Result<Self> {
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

		Ok(Client { inner: std::sync::Arc::new(inner) })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub(crate) fn get_object<T>(&self, url: reqwest::Url) -> GetObjectFuture<T> where T: serde::de::DeserializeOwned + 'static {
		let inner = self.inner.clone();

		let url_clone = url.clone();
		let builder = move ||
			inner.get(url_clone.clone())
			.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone());

		async move {
			let (response, url) = send(builder, url, false).await?;
			Ok(json(response, url).await?)
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn get_zip(&self, url: reqwest::Url, range: Option<String>) -> GetZipFuture {
		let inner = self.inner.clone();

		let is_range_request = range.is_some();

		let url_clone = url.clone();
		let builder = move || {
			let builder = inner.get(url_clone.clone());

			let builder =
				if let Some(range) = &range {
					builder.header(reqwest::header::RANGE, &**range)
				}
				else {
					builder
				};

			builder.header(reqwest::header::ACCEPT, APPLICATION_ZIP.clone())
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

		let url_clone = url.clone();
		let builder = move ||
			inner.head(url_clone.clone())
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
		// Separate inner fn so that the existential type is independent of B
		async fn post_object_inner<T>(
			inner: std::sync::Arc<reqwest::r#async::Client>,
			body: Result<String, serde_urlencoded::ser::Error>,
			url: reqwest::Url,
		) -> crate::Result<(T, reqwest::Url)> where T: serde::de::DeserializeOwned + 'static {
			let body = match body {
				Ok(body) => body,
				Err(err) => return Err(crate::ErrorKind::Serialize(url, err).into()),
			};

			let url_clone = url.clone();
			let builder = move ||
				inner.post(url_clone.clone())
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

pub(crate) existential type GetObjectFuture<T>: std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;
pub(crate) existential type GetZipFuture: std::future::Future<Output = crate::Result<(reqwest::r#async::Response, reqwest::Url)>> + 'static;
pub(crate) existential type HeadZipFuture: std::future::Future<Output = crate::Result<(reqwest::r#async::Response, reqwest::Url)>> + 'static;
pub(crate) existential type PostObjectFuture<T>: std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;

/// A login failure response.
#[derive(Debug, serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn send(
	mut builder: impl FnMut() -> reqwest::r#async::RequestBuilder,
	url: reqwest::Url,
	is_range_request: bool,
) -> crate::Result<(reqwest::r#async::Response, reqwest::Url)> {
	match url.host_str() {
		Some(host) if WHITELISTED_HOSTS.contains(host) => (),
		_ => return Err(crate::ErrorKind::NotWhitelistedHost(url).into()),
	};

	#[cfg_attr(not(windows), allow(clippy::never_loop))]
	let response = loop {
		let builder = builder();

		match futures_util::compat::Future01CompatExt::compat(builder.send()).await {
			Ok(response) => break response,
			Err(err) => {
				// native-tls sometimes fails with SEC_E_MESSAGE_ALTERED when using schannel. Retry to work around it.
				#[cfg(windows)]
				{
					use std::error::Error;

					if let Some(err) = err.source() {
						if let Some(err) = err.downcast_ref::<std::io::Error>() {
							if let Some(err) = err.get_ref() {
								if let Some(err) = err.downcast_ref::<native_tls::Error>() {
									// native_tls::Error doesn't impl Error::source, and its Error::cause impl forwards to the inner std::io::Error's cause
									// rather than the std::io::Error itself. Since it's an OS error and not a Custom error, the cause is always None.
									//
									// So check its stringified value instead.
									//
									// The full error string contains the HRESULT message string provided by FormatMessageW. Since this is localized,
									// only check the suffix generated by std::io::Error which is always in English.
									if err.to_string().ends_with(&format!(" (os error {})", winapi::shared::winerror::SEC_E_MESSAGE_ALTERED)) {
										eprintln!("Retrying request to {} because of transient SEC_E_MESSAGE_ALTERED error", url);
										continue;
									}
								}
							}
						}
					}
				}

				return Err(crate::ErrorKind::HTTP(url, err).into())
			},
		}
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

async fn json<T>(mut response: reqwest::r#async::Response, url: reqwest::Url) -> crate::Result<(T, reqwest::Url)>
	where T: serde::de::DeserializeOwned + 'static
{
	let url = expect_content_type(&response, url, &APPLICATION_JSON)?;
	match futures_util::compat::Future01CompatExt::compat(response.json()).await {
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
