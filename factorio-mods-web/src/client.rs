use ::futures::{ future, Future, IntoFuture };

/// Wraps a `reqwest::unstable::async::Client` to only allow limited operations on it.
#[derive(Debug)]
pub struct Client {
	client: ::reqwest::unstable::async::Client,
}

impl Client {
	/// Creates a new `Client` object.
	pub fn new(
		builder: Option<::reqwest::unstable::async::ClientBuilder>,
		handle: ::tokio_core::reactor::Handle,
	) -> ::Result<Self> {
		use ::error::ResultExt;

		let mut builder = match builder {
			Some(builder) => builder,
			None => ::reqwest::unstable::async::ClientBuilder::new().chain_err(|| "Could not create HTTP client")?,
		};

		let client =
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
			.chain_err(|| "Could not create HTTP client")?;

		Ok(Client { client })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub fn get_object<'a, T>(&'a self, url: ::reqwest::Url) -> impl Future<Item = (T, ::reqwest::Url), Error = ::Error> + 'a
		where T: Send + 'a, for<'de> T: ::serde::Deserialize<'de> {

		match self.client.get(url.clone()) {
			Ok(mut builder) => {
				builder.header(::reqwest::header::Accept::json());

				future::Either::A(
					self.send(builder, url)
					.and_then(|(response, url)| json(response, url)))
			},

			Err(err) =>
				future::Either::B(future::err(::ErrorKind::HTTP(url, err).into())),
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip<'a>(&'a self, url: ::reqwest::Url) -> impl Future<Item = (::reqwest::unstable::async::Response, ::reqwest::Url), Error = ::Error> + 'a {
		match self.client.get(url.clone()) {
			Ok(mut builder) => {
				builder.header(ACCEPT_APPLICATION_ZIP.clone());

				future::Either::A(
					self.send(builder, url)
					.and_then(|(response, url)| expect_content_type(response, url, &APPLICATION_ZIP)))
			},

			Err(err) =>
				future::Either::B(future::err(::ErrorKind::HTTP(url, err).into())),
		}
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<'a, B, T>(&'a self, url: ::reqwest::Url, body: &B) -> Box<Future<Item = (T, ::reqwest::Url), Error = ::Error> + 'a>
		where B: ::serde::Serialize, T: Send + 'a, for<'de> T: ::serde::Deserialize<'de> {

		// TODO: Box because of bug in `conservative_impl_trait` that somehow requires `body` to be `'a` too
		// Repro: http://play.integer32.com/?gist=c4baba83cc00a45ddeed9b799222358f&version=nightly
		// which works when changed to not use impl trait: http://play.integer32.com/?gist=cf52c03896a6b24d48d26c365ea6a5a6&version=nightly

		match self.client.post(url.clone()) {
			Ok(mut builder) => {
				match builder.header(::reqwest::header::Accept::json()).form(body) {
					Ok(_) =>
						Box::new(
							self.send(builder, url)
							.and_then(|(response, url)| json(response, url))),

					Err(err) =>
						Box::new(future::err(::ErrorKind::HTTP(url, err).into())),
				}
			},

			Err(err) =>
				Box::new(future::err(::ErrorKind::HTTP(url, err).into())),
		}
	}

	fn send<'a>(
		&'a self,
		mut builder: ::reqwest::unstable::async::RequestBuilder,
		url: ::reqwest::Url,
	) -> impl Future<Item = (::reqwest::unstable::async::Response, ::reqwest::Url), Error = ::Error> + 'a {
		builder.header(USER_AGENT.clone());

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
}

lazy_static! {
	static ref WHITELISTED_HOSTS: ::std::collections::HashSet<&'static str> = vec![
		"mods.factorio.com",
		"mods-data.factorio.com",
	].into_iter().collect();
	static ref USER_AGENT: ::reqwest::header::UserAgent = ::reqwest::header::UserAgent::new(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));
	static ref APPLICATION_ZIP: ::reqwest::mime::Mime = "application/zip".parse().unwrap();
	static ref ACCEPT_APPLICATION_ZIP: ::reqwest::header::Accept = ::reqwest::header::Accept(vec![::reqwest::header::qitem(APPLICATION_ZIP.clone())]);
}

/// A login failure response.
#[derive(Debug, ::serde_derive::Deserialize)]
struct LoginFailureResponse {
	message: String,
}

fn json<T>(response: ::reqwest::unstable::async::Response, url: ::reqwest::Url) -> impl Future<Item = (T, ::reqwest::Url), Error = ::Error>
	where T: Send, for<'de> T: ::serde::Deserialize<'de> {

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
