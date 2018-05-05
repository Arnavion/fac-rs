#![cfg_attr(feature = "cargo-clippy", allow(
	single_match_else,
))]

use ::futures::{ future, Future };

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

		::async_block! {
			builder.header(::reqwest::header::Accept::json());
			let (response, url) = ::await!(send(builder, url))?;
			Ok(::await!(json(response, url))?)
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip(&self, url: ::reqwest::Url) -> impl Future<Item = (::reqwest::unstable::async::Response, ::reqwest::Url), Error = ::Error> + 'static {
		let mut builder = self.inner.get(url.clone());

		::async_block! {
			builder.header(ACCEPT_APPLICATION_ZIP.clone());
			let (response, url) = ::await!(send(builder, url))?;
			let url = expect_content_type(&response, url, &APPLICATION_ZIP)?;
			Ok((response, url))
		}
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<B, T>(&self, url: ::reqwest::Url, body: B) -> Box<Future<Item = (T, ::reqwest::Url), Error = ::Error>>
		where B: ::serde::Serialize, T: ::serde::de::DeserializeOwned + 'static {

		// TODO: `Box` and `'static` because of bug in impl trait that somehow requires `body` to be `'static` too
		// https://github.com/rust-lang/rust/issues/42940

		let mut builder = self.inner.post(url.clone());

		let body = match ::serde_urlencoded::to_string(body) {
			Ok(body) => body,
			Err(err) => return Box::new(future::err(::ErrorKind::Serialize(url, err).into())),
		};

		Box::new(::async_block! {
			// TODO: Workaround for https://github.com/seanmonstar/reqwest/issues/214
			// builder.header(::reqwest::header::Accept::json()).form(&body);
			builder
			.header(::reqwest::header::Accept::json())
			.header(::reqwest::header::ContentType::form_url_encoded())
			.header(::reqwest::header::ContentLength(body.len() as u64))
			.body(body);

			let (response, url) = ::await!(send(builder, url))?;
			Ok(::await!(json(response, url))?)
		})
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
	::async_block! {
		let is_whitelisted_host = match url.host_str() {
			Some(host) if WHITELISTED_HOSTS.contains(host) => true,
			_ => false,
		};

		ensure!(is_whitelisted_host, ::ErrorKind::NotWhitelistedHost(url));

		let response = match ::await!(builder.send()) {
			Ok(response) => response,
			Err(err) => bail!(::ErrorKind::HTTP(url, err)),
		};

		// TODO: Workaround for https://github.com/rust-lang/rust/issues/44197
		let status = response.status();
		match status {
			::reqwest::StatusCode::Ok => Ok((response, url)),

			::reqwest::StatusCode::Unauthorized => {
				let (object, _): (LoginFailureResponse, _) = ::await!(json(response, url))?;
				bail!(::ErrorKind::LoginFailure(object.message));
			},

			::reqwest::StatusCode::Found => bail!(::ErrorKind::NotWhitelistedHost(url)),

			code => bail!(::ErrorKind::StatusCode(url, code)),
		}
	}
}

fn json<T>(mut response: ::reqwest::unstable::async::Response, url: ::reqwest::Url) -> impl Future<Item = (T, ::reqwest::Url), Error = ::Error> + 'static
	where T: ::serde::de::DeserializeOwned + 'static {

	::async_block! {
		let url = expect_content_type(&response, url, &::reqwest::mime::APPLICATION_JSON)?;
		match ::await!(response.json()) {
			Ok(object) => Ok((object, url)),
			Err(err) => bail!(::ErrorKind::HTTP(url, err)),
		}
	}
}

fn expect_content_type(
	response: &::reqwest::unstable::async::Response,
	url: ::reqwest::Url,
	expected_mime: &::reqwest::mime::Mime,
) -> ::Result<::reqwest::Url> {
	match response.headers().get() {
		Some(&::reqwest::header::ContentType(ref mime)) if mime == expected_mime =>
			Ok(url),
		Some(&::reqwest::header::ContentType(ref mime)) =>
			bail!(::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {}", mime))),
		None =>
			bail!(::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string())),
	}
}
