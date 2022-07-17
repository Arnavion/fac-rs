/// Wraps a `hyper::Client` to only allow limited operations on it.
#[derive(Debug)]
pub(crate) struct Client {
	inner: std::sync::Arc<ClientInner>,
}

impl Client {
	/// Creates a new `Client` object.
	pub(crate) fn new() -> Self {
		let connector = hyper_tls::HttpsConnector::new();
		let inner = hyper::Client::builder().build(connector);
		let user_agent = http::HeaderValue::from_static(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));

		Client {
			inner: std::sync::Arc::new(ClientInner {
				inner,
				user_agent,
			}),
		}
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub(crate) fn get_object<T>(&self, url: url::Url) -> impl std::future::Future<Output = Result<(T, url::Url), crate::Error>>
	where
		T: serde::de::DeserializeOwned + 'static,
	{
		let inner = self.inner.clone();

		async {
			let request = {
				let mut request = http::Request::new(Default::default());
				*request.method_mut() = http::Method::GET;
				*request.uri_mut() = match url.to_string().parse() {
					Ok(uri) => uri,
					Err(err) => return Err(crate::Error::ParseUri(url, err)),
				};
				request
			};

			let (response, url) = inner.send(request, None, &APPLICATION_JSON, url).await?;
			json(response, url).await
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn get_zip(&self, url: url::Url, range: Option<http::HeaderValue>) ->
		impl std::future::Future<Output = Result<(http::Response<hyper::Body>, url::Url), crate::Error>>
	{
		let inner = self.inner.clone();

		async {
			let request = {
				let mut request = http::Request::new(Default::default());
				*request.method_mut() = http::Method::GET;
				*request.uri_mut() = match url.to_string().parse() {
					Ok(uri) => uri,
					Err(err) => return Err(crate::Error::ParseUri(url, err)),
				};
				request
			};

			let (response, url) = inner.send(request, range, &APPLICATION_ZIP, url).await?;
			let url = expect_content_type(&response, url, [&APPLICATION_OCTET_STREAM, &APPLICATION_ZIP])?;
			Ok((response, url))
		}
	}

	/// HEADs the given URL using the given client, and returns an application/zip response.
	pub(crate) fn head_zip(&self, url: url::Url) -> impl std::future::Future<Output = Result<(http::Response<hyper::Body>, url::Url), crate::Error>> {
		let inner = self.inner.clone();

		async {
			let request = {
				let mut request = http::Request::new(Default::default());
				*request.method_mut() = http::Method::HEAD;
				*request.uri_mut() = match url.to_string().parse() {
					Ok(uri) => uri,
					Err(err) => return Err(crate::Error::ParseUri(url, err)),
				};
				request
			};

			let (response, url) = inner.send(request, None, &APPLICATION_ZIP, url).await?;
			let url = expect_content_type(&response, url, [&APPLICATION_OCTET_STREAM, &APPLICATION_ZIP])?;
			Ok((response, url))
		}
	}

	// TODO: Would like to return `impl std::future::Future<Output = Result<(T, url::Url), crate::Error>>`,
	// but https://github.com/rust-lang/rust/issues/42940 prevents it.
	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub(crate) fn post_object<B, T>(&self, url: url::Url, body: &B) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(T, url::Url), crate::Error>>>>
		where B: serde::Serialize, T: serde::de::DeserializeOwned + 'static
	{
		let body = serde_urlencoded::to_string(body);

		let inner = self.inner.clone();

		Box::pin(async {
			let request = {
				let body = match body {
					Ok(body) => body,
					Err(err) => return Err(crate::Error::Serialize(url, err)),
				};
				let mut request = http::Request::new(body.into());
				*request.method_mut() = http::Method::POST;
				*request.uri_mut() = match url.to_string().parse() {
					Ok(uri) => uri,
					Err(err) => return Err(crate::Error::ParseUri(url, err)),
				};
				request.headers_mut().insert(http::header::CONTENT_TYPE, WWW_FORM_URL_ENCODED.clone());
				request
			};

			let (response, url) = inner.send(request, None, &APPLICATION_JSON, url).await?;
			json(response, url).await
		})
	}
}

static WHITELISTED_HOSTS: once_cell::sync::Lazy<std::collections::BTreeSet<&'static str>> =
	once_cell::sync::Lazy::new(|| [
		"auth.factorio.com",
		"dl-mod.factorio.com",
		"mods.factorio.com",
	].into_iter().collect());

static APPLICATION_JSON: once_cell::sync::Lazy<http::HeaderValue> =
	once_cell::sync::Lazy::new(|| http::HeaderValue::from_static("application/json"));

static APPLICATION_OCTET_STREAM: once_cell::sync::Lazy<http::HeaderValue> =
	once_cell::sync::Lazy::new(|| http::HeaderValue::from_static("application/octet-stream"));

static APPLICATION_ZIP: once_cell::sync::Lazy<http::HeaderValue> =
	once_cell::sync::Lazy::new(|| http::HeaderValue::from_static("application/zip"));

static WWW_FORM_URL_ENCODED: once_cell::sync::Lazy<http::HeaderValue> =
	once_cell::sync::Lazy::new(|| http::HeaderValue::from_static("application/x-www-form-urlencoded"));

#[derive(Debug)]
struct ClientInner {
	inner: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
	user_agent: http::HeaderValue,
}

impl ClientInner {
	fn send(
		self: std::sync::Arc<Self>,
		request: http::Request<hyper::Body>,
		range: Option<http::HeaderValue>,
		accept: &'static http::HeaderValue,
		url: url::Url,
	) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(http::Response<hyper::Body>, url::Url), crate::Error>>>> {
		async fn send_inner(
			this: std::sync::Arc<ClientInner>,
			mut request: http::Request<hyper::Body>,
			range: Option<http::HeaderValue>,
			accept: &'static http::HeaderValue,
			url: url::Url,
		) -> Result<(http::Response<hyper::Body>, url::Url), crate::Error> {
			if !matches!(url.host_str(), Some(host) if WHITELISTED_HOSTS.contains(host)) {
				return Err(crate::Error::NotWhitelistedHost(url));
			}

			{
				let headers = request.headers_mut();
				headers.insert(http::header::ACCEPT, accept.clone());
				headers.insert(http::header::USER_AGENT, this.user_agent.clone());
				if let Some(range) = &range {
					request.headers_mut().insert(http::header::RANGE, range.clone());
				}
			}

			let response = match this.inner.request(request).await {
				Ok(response) => response,
				Err(err) => return Err(crate::Error::Http(url, err)),
			};

			match response.status() {
				http::StatusCode::OK if range.is_none() => Ok((response, url)),

				http::StatusCode::PARTIAL_CONTENT if range.is_some() => Ok((response, url)),

				http::StatusCode::FOUND => {
					let location = match response.headers().get(http::header::LOCATION) {
						Some(location) => location,
						None => return Err(crate::Error::MalformedResponse(url, "No Location header".to_owned())),
					};
					let location = match location.to_str() {
						Ok(location) => location,
						Err(err) => return Err(crate::Error::MalformedResponse(url, format!("Malformed Location header: {err}"))),
					};
					let location = match url.join(location) {
						Ok(location) => location,
						Err(err) => return Err(crate::Error::MalformedResponse(url, format!("Malformed Location header: {err}"))),
					};

					let request = {
						let mut request = http::Request::new(Default::default());
						*request.method_mut() = http::Method::GET;
						*request.uri_mut() = match location.to_string().parse() {
							Ok(uri) => uri,
							Err(err) => return Err(crate::Error::ParseUri(url, err)),
						};
						request
					};

					this.send(request, range, accept, location).await
				},

				http::StatusCode::UNAUTHORIZED => {
					let (object, _): (LoginFailureResponse, _) = json(response, url).await?;
					Err(crate::Error::LoginFailure(object.message))
				},

				code => Err(crate::Error::StatusCode(url, code)),
			}
		}

		Box::pin(send_inner(self, request, range, accept, url))
	}
}

/// A login failure response.
#[derive(Debug, serde::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

async fn json<T>(response: http::Response<hyper::Body>, url: url::Url) -> Result<(T, url::Url), crate::Error>
	where T: serde::de::DeserializeOwned + 'static
{
	let url = expect_content_type(&response, url, [&APPLICATION_JSON])?;
	let response = response.into_body();
	let response = match hyper::body::aggregate(response).await {
		Ok(response) => response,
		Err(err) => return Err(crate::Error::Http(url, err)),
	};

	// `serde_json::from_reader` on a reader made from `hyper::body::aggregate`'s result is very slow in debug builds,
	// due to `serde_json`'s propensity to do many 1-byte reads, each of which involves expensive iterating over
	// the elements of the `VecDeque<bytes::Bytes>` that `hyper::body::aggregate`'s result contains.
	//
	// For the sake of tests, debug builds read the response into a slice first.
	let object =
		if cfg!(debug_assertions) {
			let mut buf = vec![];
			std::io::Read::read_to_end(&mut bytes::Buf::reader(response), &mut buf).unwrap();
			serde_json::from_slice(&buf)
		}
		else {
			serde_json::from_reader(bytes::Buf::reader(response))
		};
	let object = match object {
		Ok(object) => object,
		Err(err) => return Err(crate::Error::Deserialize(url, err)),
	};
	Ok((object, url))
}

fn expect_content_type<'a, I, T>(
	response: &http::Response<hyper::Body>,
	url: url::Url,
	expected_mime: I,
) -> Result<url::Url, crate::Error>
where
	I: IntoIterator<Item = &'a T>,
	T: std::ops::Deref<Target = http::HeaderValue> + 'a,
{
	let mime = match response.headers().get(http::header::CONTENT_TYPE) {
		Some(mime) => mime,
		None => return Err(crate::Error::MalformedResponse(url, "No Content-Type header".to_owned())),
	};

	if !expected_mime.into_iter().any(|expected_mime| mime == **expected_mime) {
		return Err(crate::Error::MalformedResponse(url, format!("Unexpected Content-Type header: {mime:?}")));
	}

	Ok(url)
}
