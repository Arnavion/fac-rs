/// Wraps a `reqwest::Client` to only allow limited operations on it.
#[derive(Debug)]
pub(crate) struct Client {
	inner: reqwest::Client,
}

impl Client {
	/// Creates a new `Client` object.
	pub(crate) fn new(builder: Option<reqwest::ClientBuilder>) -> Result<Self, crate::Error> {
		let builder = builder.unwrap_or_else(reqwest::ClientBuilder::new);

		let mut default_headers = reqwest::header::HeaderMap::new();
		default_headers.insert(
			reqwest::header::USER_AGENT,
			reqwest::header::HeaderValue::from_static(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))));
		let builder = builder.default_headers(default_headers);

		let inner =
			builder
			.redirect(reqwest::redirect::Policy::custom(|attempt| {
				if matches!(attempt.url().host_str(), Some(host) if WHITELISTED_HOSTS.contains(host)) {
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
	pub(crate) fn get_object<T>(&self, url: reqwest::Url) -> impl std::future::Future<Output = Result<(T, reqwest::Url), crate::Error>>
	where
		T: serde::de::DeserializeOwned + 'static,
	{
		let builder =
			self.inner.get(url.clone())
			.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone());

		async move {
			let (response, url) = send(builder, url, false).await?;
			Ok(json(response, url).await?)
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn get_zip(&self, url: reqwest::Url, range: Option<&str>) -> impl std::future::Future<Output = Result<(reqwest::Response, reqwest::Url), crate::Error>> {
		let (builder, is_range_request) = {
			let builder = self.inner.get(url.clone());

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
	pub(crate) fn head_zip(&self, url: reqwest::Url) -> impl std::future::Future<Output = Result<(reqwest::Response, reqwest::Url), crate::Error>> {
		let builder =
			self.inner.head(url.clone())
			.header(reqwest::header::ACCEPT, APPLICATION_ZIP.clone());

		async move {
			let (response, url) = send(builder, url, false).await?;
			let url = expect_content_type(&response, url, &APPLICATION_ZIP)?;
			Ok((response, url))
		}
	}

	// TODO: Would like to return `impl std::future::Future<Output = Result<(T, reqwest::Url), crate::Error>>`,
	// but https://github.com/rust-lang/rust/issues/42940 prevents it.
	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub(crate) fn post_object<B, T>(&self, url: reqwest::Url, body: &B) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(T, reqwest::Url), crate::Error>>>>
		where B: serde::Serialize, T: serde::de::DeserializeOwned + 'static
	{
		let builder = self.inner.post(url.clone());
		let body = serde_urlencoded::to_string(body);
		Box::pin(async move {
			let body = match body {
				Ok(body) => body,
				Err(err) => return Err(crate::ErrorKind::Serialize(url, err).into()),
			};

			let builder =
				builder
				.header(reqwest::header::ACCEPT, APPLICATION_JSON.clone())
				.header(reqwest::header::CONTENT_TYPE, WWW_FORM_URL_ENCODED.clone())
				.body(body);

			let (response, url) = send(builder, url, false).await?;
			Ok(json(response, url).await?)
		})
	}
}

static WHITELISTED_HOSTS: once_cell::sync::Lazy<std::collections::HashSet<&'static str>> =
	once_cell::sync::Lazy::new(|| [
		"auth.factorio.com",
		"direct.mods-data.factorio.com",
		"mods.factorio.com",
		"mods-data.factorio.com",
	].iter().copied().collect());

static APPLICATION_JSON: once_cell::sync::Lazy<reqwest::header::HeaderValue> =
	once_cell::sync::Lazy::new(|| reqwest::header::HeaderValue::from_static("application/json"));

static APPLICATION_ZIP: once_cell::sync::Lazy<reqwest::header::HeaderValue> =
	once_cell::sync::Lazy::new(|| reqwest::header::HeaderValue::from_static("application/zip"));

static WWW_FORM_URL_ENCODED: once_cell::sync::Lazy<reqwest::header::HeaderValue> =
	once_cell::sync::Lazy::new(|| reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"));

/// A login failure response.
#[derive(Debug, serde::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn send(
	builder: reqwest::RequestBuilder,
	url: reqwest::Url,
	is_range_request: bool,
) -> Result<(reqwest::Response, reqwest::Url), crate::Error> {
	if !matches!(url.host_str(), Some(host) if WHITELISTED_HOSTS.contains(host)) {
		return Err(crate::ErrorKind::NotWhitelistedHost(url).into());
	}

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

async fn json<T>(response: reqwest::Response, url: reqwest::Url) -> Result<(T, reqwest::Url), crate::Error>
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
) -> Result<reqwest::Url, crate::Error> {
	let mime = match response.headers().get(reqwest::header::CONTENT_TYPE) {
		Some(mime) => mime,
		None => return Err(crate::ErrorKind::MalformedResponse(url, "No Content-Type header".to_owned()).into()),
	};

	if mime != expected_mime {
		return Err(crate::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {:?}", mime)).into());
	}

	Ok(url)
}
