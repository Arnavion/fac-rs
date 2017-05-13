/// Wraps a `reqwest::Client` to only allow limited operations on it.
#[derive(Debug)]
pub struct Client {
	client: ::reqwest::Client,
}

impl Client {
	/// Creates a new `Client` object.
	pub fn new(client: Option<::reqwest::Client>) -> ::Result<Client> {
		let mut client = match client {
			Some(client) => client,
			None => ::error::ResultExt::chain_err(::reqwest::Client::new(), || "Could not create HTTP client")?,
		};

		client.redirect(::reqwest::RedirectPolicy::custom(|attempt| {
			if match attempt.url().host_str() {
				Some(host) if HOSTS_TO_ACCEPT_REDIRECTS_TO.contains(host) => true,
				_ => false,
			} {
				attempt.follow()
			}
			else {
				attempt.stop()
			}
		}));

		Ok(Client { client })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub fn get_object<T>(&self, url: ::reqwest::Url) -> ::Result<T> where for<'de> T: ::serde::Deserialize<'de> {
		loop {
			let request = self.client.get(url.clone()).header(USER_AGENT.clone()).header(APPLICATION_JSON.clone());
			match send(request, url.clone()) {
				Ok(response) => {
					let object = json(response, url)?;
					return Ok(object);
				},

				Err(err) => {
					match *err.kind() {
						::ErrorKind::HTTP(_, ref reqwest_err) => {
							if let Some(reqwest_err_cause) = reqwest_err.get_ref() {
								if let Some(&::reqwest::HyperError::Io(ref io_err)) = reqwest_err_cause.downcast_ref() {
									if let ::std::io::ErrorKind::ConnectionAborted = io_err.kind() {
										continue;
									}
								}
							}
						},

						::ErrorKind::StatusCode(_, ::reqwest::StatusCode::ServiceUnavailable) => continue,

						_ => { },
					}

					return Err(err);
				},
			}
		}
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip(&self, url: ::reqwest::Url) -> ::Result<::reqwest::Response> {
		let request = self.client.get(url.clone()).header(USER_AGENT.clone()).header(APPLICATION_ZIP.clone());
		let response = send(request, url.clone())?;

		match response.headers().get() {
			Some(&::reqwest::header::ContentType(::reqwest::mime::Mime(::reqwest::mime::TopLevel::Application, ::reqwest::mime::SubLevel::Ext(ref sublevel), _))) if sublevel == "zip" =>
				(),
			Some(&::reqwest::header::ContentType(ref mime)) =>
				bail!(::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {}", mime))),
			None =>
				bail!(::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string())),
		}

		Ok(response)
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<B, T>(&self, url: ::reqwest::Url, body: &B) -> ::Result<T>
		where B: ::serde::Serialize, for<'de> T: ::serde::Deserialize<'de> {

		let request = self.client.post(url.clone()).header(USER_AGENT.clone()).header(APPLICATION_JSON.clone()).form(body);
		let response = send(request, url.clone())?;
		let object = json(response, url)?;
		Ok(object)
	}
}

lazy_static! {
	static ref HOSTS_TO_ACCEPT_REDIRECTS_TO: ::std::collections::HashSet<&'static str> = vec![
		"mods.factorio.com",
		"mods-data.factorio.com",
	].into_iter().collect();
	static ref USER_AGENT: ::reqwest::header::UserAgent = ::reqwest::header::UserAgent(format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
	static ref APPLICATION_JSON: ::reqwest::header::Accept = ::reqwest::header::Accept::json();
	static ref APPLICATION_ZIP: ::reqwest::header::Accept =
		::reqwest::header::Accept(vec![::reqwest::header::qitem(::reqwest::mime::Mime(
			::reqwest::mime::TopLevel::Application, ::reqwest::mime::SubLevel::Ext("zip".to_string()), vec![]))]);
}

/// A login failure response.
#[derive(Debug, Deserialize)]
struct LoginFailureResponse {
	message: String,
}

fn send(request: ::reqwest::RequestBuilder, url: ::reqwest::Url) -> ::Result<::reqwest::Response> {
	let response = request.send().map_err(|err| ::ErrorKind::HTTP(url.clone(), err))?;
	Ok(match *response.status() {
		::reqwest::StatusCode::Ok =>
			response,

		::reqwest::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = json(response, url)?;
			bail!(::ErrorKind::LoginFailure(object.message))
		},

		::reqwest::StatusCode::Found =>
			bail!(::ErrorKind::UnexpectedRedirect(url)),

		code =>
			bail!(::ErrorKind::StatusCode(url, code)),
	})
}

fn json<T>(mut response: ::reqwest::Response, url: ::reqwest::Url) -> ::Result<T> where for<'de> T: ::serde::Deserialize<'de> {
	match response.headers().get() {
		Some(&::reqwest::header::ContentType(::reqwest::mime::Mime(::reqwest::mime::TopLevel::Application, ::reqwest::mime::SubLevel::Json, _))) =>
			(),
		Some(&::reqwest::header::ContentType(ref mime)) =>
			bail!(::ErrorKind::MalformedResponse(url, format!("Unexpected Content-Type header: {}", mime))),
		None =>
			bail!(::ErrorKind::MalformedResponse(url, "No Content-Type header".to_string())),
	}

	let object = response.json().map_err(|err| ::ErrorKind::HTTP(url, err))?;

	Ok(object)
}
