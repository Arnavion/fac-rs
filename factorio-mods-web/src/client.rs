use ::futures::{ future, Future, IntoFuture };

/// Wraps a `reqwest::unstable::async::Client` to only allow limited operations on it.
#[derive(Debug)]
pub struct Client {
	inner: ::reqwest::unstable::async::Client,
}

impl Client {
	/// Creates a new `Client` object.
	#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))] // reqwest::ClientBuilder::build violates API guidelines. Don't perpetuate it.
	pub fn new(
		builder: Option<::reqwest::unstable::async::ClientBuilder>,
		handle: ::tokio_core::reactor::Handle,
	) -> ::Result<Self> {
		let mut builder = builder.unwrap_or_else(::reqwest::unstable::async::ClientBuilder::new);

		let mut default_headers = ::reqwest::header::Headers::new();
		default_headers.set(::reqwest::header::UserAgent::new(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))));
		builder.default_headers(default_headers);

		let inner =
			builder
			.redirect(::reqwest::RedirectPolicy::custom(|attempt| {
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
			.build(&handle)
			.map_err(::ErrorKind::CreateClient)?;

		Ok(Client { inner })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub fn get_object<T>(&self, url: ::reqwest::Url) -> impl Future<Item = (T, ::reqwest::Url), Error = ::Error> + 'static
		where T: ::serde::de::DeserializeOwned + 'static {

		let mut builder = self.inner.get(url.clone());
		builder.header(::reqwest::header::Accept::json());
		send(builder, url)
		.and_then(|(response, url)| json(response, url))
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip(&self, url: ::reqwest::Url) -> impl Future<Item = (::reqwest::unstable::async::Response, ::reqwest::Url), Error = ::Error> + 'static {
		let mut builder = self.inner.get(url.clone());
		builder.header(ACCEPT_APPLICATION_ZIP.clone());
		send(builder, url)
		.and_then(|(response, url)| expect_content_type(response, url, &APPLICATION_ZIP))
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<B, T>(&self, url: ::reqwest::Url, body: B) -> Box<Future<Item = (T, ::reqwest::Url), Error = ::Error>>
		where B: ::serde::Serialize, T: ::serde::de::DeserializeOwned + 'static {

		// TODO: `Box` and `'static` because of bug in `conservative_impl_trait` that somehow requires `body` to be `'static` too
		// https://github.com/rust-lang/rust/issues/42940

		let mut builder = self.inner.post(url.clone());

		// TODO: Workaround for https://github.com/seanmonstar/reqwest/issues/214
		// builder.header(::reqwest::header::Accept::json()).form(&body);
		let body = match ::serde_urlencoded::to_string(body) {
			Ok(body) => body,
			Err(err) => return Box::new(future::err(::ErrorKind::Serialize(url, err).into())),
		};
		builder
		.header(::reqwest::header::Accept::json())
		.header(::reqwest::header::ContentType::form_url_encoded())
		.header(::reqwest::header::ContentLength(body.len() as u64))
		.body(body);

		Box::new(
			send(builder, url)
			.and_then(|(response, url)| json(response, url)))
	}
}

lazy_static! {
	static ref WHITELISTED_HOSTS: ::std::collections::HashSet<&'static str> = vec![
		"auth.factorio.com",
		"mods.factorio.com",
		"mods-data.factorio.com",
	].into_iter().collect();
	static ref APPLICATION_ZIP: ::reqwest::mime::Mime = "application/zip".parse().unwrap();
	static ref ACCEPT_APPLICATION_ZIP: ::reqwest::header::Accept = ::reqwest::header::Accept(vec![::reqwest::header::qitem(APPLICATION_ZIP.clone())]);
}

/// A login failure response.
#[derive(Debug, ::serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

fn send(
	mut builder: ::reqwest::unstable::async::RequestBuilder,
	url: ::reqwest::Url,
) -> impl Future<Item = (::reqwest::unstable::async::Response, ::reqwest::Url), Error = ::Error> + 'static {
	let is_whitelisted_host = match url.host_str() {
		Some(host) if WHITELISTED_HOSTS.contains(host) => true,
		_ => false,
	};

	if !is_whitelisted_host {
		return future::Either::A(future::err(::ErrorKind::NotWhitelistedHost(url).into()))
	}

	future::Either::B(
		builder.send()
		.then(move |response| match response {
			Ok(response) => match response.status() {
				::reqwest::StatusCode::Ok =>
					future::Either::A(future::ok((response, url))),

				::reqwest::StatusCode::Unauthorized =>
					future::Either::B(
						json(response, url)
						.and_then(|(object, _): (LoginFailureResponse, _)|
							future::err(::Error::from(::ErrorKind::LoginFailure(object.message))))),

				::reqwest::StatusCode::Found =>
					future::Either::A(future::err(::ErrorKind::NotWhitelistedHost(url).into())),

				code =>
					future::Either::A(future::err(::ErrorKind::StatusCode(url, code).into())),
			},

			Err(err) =>
				future::Either::A(future::err(::ErrorKind::HTTP(url, err).into())),
		}))
}

fn json<T>(response: ::reqwest::unstable::async::Response, url: ::reqwest::Url) -> impl Future<Item = (T, ::reqwest::Url), Error = ::Error> + 'static
	where T: ::serde::de::DeserializeOwned + 'static {

	expect_content_type(response, url, &::reqwest::mime::APPLICATION_JSON)
	.into_future()
	.and_then(|(mut response, url)|
		response.json()
		.then(|object| match object {
			Ok(object) => Ok((object, url)),
			Err(err) => Err(::ErrorKind::HTTP(url, err).into()),
		}))
}

fn expect_content_type(
	response: ::reqwest::unstable::async::Response,
	url: ::reqwest::Url,
	expected_mime: &::reqwest::mime::Mime,
) -> ::Result<(::reqwest::unstable::async::Response, ::reqwest::Url)> {
	match response.headers().get() {
		Some(&::reqwest::header::ContentType(ref mime)) if mime == expected_mime =>
			(),
		Some(&::reqwest::header::ContentType(ref mime)) =>
			bail!(::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {}", mime))),
		None =>
			bail!(::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string())),
	};

	Ok((response, url))
}
