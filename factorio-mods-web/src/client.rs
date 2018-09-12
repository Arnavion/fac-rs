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
	pub fn get_object<T>(&self, url: reqwest::Url) -> impl std::future::Future<Output = crate::Result<(T, reqwest::Url)>> + 'static where T: serde::de::DeserializeOwned + 'static {
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
	pub fn post_object<B, T>(&self, url: reqwest::Url, body: &B) -> std::future::LocalFutureObj<'static, crate::Result<(T, reqwest::Url)>>
		where B: serde::Serialize, T: serde::de::DeserializeOwned + 'static
	{
		// TODO: Replace return type with PostObjectFuture

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

		std::future::LocalFutureObj::new(std::pin::PinBox::new(inner(url, builder, body)))
	}
}

lazy_static! {
	static ref WHITELISTED_HOSTS: std::collections::HashSet<&'static str> = vec![
		"auth.factorio.com",
		"mods.factorio.com",
		"mods-data.factorio.com",
	].into_iter().collect();
	static ref APPLICATION_JSON: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/json");
	static ref APPLICATION_ZIP: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/zip");
	static ref WWW_FORM_URL_ENCODED: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded");
}

// TODO: Use existential type when https://github.com/rust-lang/rust/issues/53443 is fixed
// pub(crate) existential type GetObjectFuture<T>: Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;
// existential type PostObjectFuture<T>: Future<Output = crate::Result<(T, reqwest::Url)>> + 'static;

/// A login failure response.
#[derive(Debug, serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn send(
	builder: reqwest::r#async::RequestBuilder,
	url: reqwest::Url,
) -> crate::Result<(reqwest::r#async::Response, reqwest::Url)> {
	let is_whitelisted_host = match url.host_str() {
		Some(host) if WHITELISTED_HOSTS.contains(host) => true,
		_ => false,
	};

	if !is_whitelisted_host {
		error_chain::bail!(crate::ErrorKind::NotWhitelistedHost(url));
	}

	let response = match await!(futures_util::compat::Future01CompatExt::compat(builder.send())) {
		Ok(response) => response,
		Err(err) => error_chain::bail!(crate::ErrorKind::HTTP(url, err)),
	};

	match response.status() {
		reqwest::StatusCode::OK => Ok((response, url)),

		reqwest::StatusCode::UNAUTHORIZED => {
			let (object, _): (LoginFailureResponse, _) = await!(json(response, url))?;
			error_chain::bail!(crate::ErrorKind::LoginFailure(object.message));
		},

		reqwest::StatusCode::FOUND => error_chain::bail!(crate::ErrorKind::NotWhitelistedHost(url)),

		code => error_chain::bail!(crate::ErrorKind::StatusCode(url, code)),
	}
}

async fn json<T>(mut response: reqwest::r#async::Response, url: reqwest::Url) -> crate::Result<(T, reqwest::Url)>
	where T: serde::de::DeserializeOwned + 'static
{
	let url = expect_content_type(&response, url, &APPLICATION_JSON)?;
	match await!(futures_util::compat::Future01CompatExt::compat(response.json())) {
		Ok(object) => Ok((object, url)),
		Err(err) => error_chain::bail!(crate::ErrorKind::HTTP(url, err)),
	}
}

fn expect_content_type(
	response: &reqwest::r#async::Response,
	url: reqwest::Url,
	expected_mime: &reqwest::header::HeaderValue,
) -> crate::Result<reqwest::Url> {
	let mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
		Some(mime) => mime,
		None => error_chain::bail!(crate::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string())),
	};

	if mime != expected_mime {
		error_chain::bail!(crate::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {:?}", mime)));
	}

	Ok(url)
}
