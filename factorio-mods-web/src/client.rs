/// Wraps a `reqwest::Client` to only allow limited operations on it.
#[derive(Debug)]
pub struct Client {
	client: ::reqwest::Client,
}

impl Client {
	/// Creates a new `Client` object.
	pub fn new(client: Option<::reqwest::Client>, base_url_host: String) -> ::Result<Client> {
		let mut client = match client {
			Some(client) => client,
			None => ::reqwest::Client::new()?,
		};

		client.redirect(::reqwest::RedirectPolicy::custom(move |url, _| {
			if let Some(host) = url.host_str() {
				if host != base_url_host {
					return Ok(true);
				}
			}

			Ok(url.path() != "/login")
		}));

		Ok(Client { client: client })
	}

	/// GETs the given URL using the given client, and deserializes the response as a JSON object.
	pub fn get_object<T>(&self, url: ::reqwest::Url) -> ::Result<T> where T: ::serde::Deserialize {
		let request = self.client.get(url).header(USER_AGENT.clone()).header(APPLICATION_JSON.clone());
		let response = send(request)?;
		let object = json(response)?;
		Ok(object)
	}

	/// GETs the given URL using the given client, and returns an application/zip response.
	pub fn get_zip(&self, url: ::reqwest::Url) -> ::Result<::reqwest::Response> {
		let request = self.client.get(url).header(USER_AGENT.clone()).header(APPLICATION_ZIP.clone());
		let response = send(request)?;

		match response.headers().get() {
			Some(&::reqwest::header::ContentType(::mime::Mime(::mime::TopLevel::Application, ::mime::SubLevel::Ext(ref sublevel), _))) if sublevel == "zip" =>
				(),
			Some(&::reqwest::header::ContentType(ref mime)) =>
				bail!(::ErrorKind::MalformedResponse(format!("Unexpected Content-Type header: {}", mime))),
			None =>
				bail!(::ErrorKind::MalformedResponse("No Content-Type header".to_string())),
		}

		Ok(response)
	}

	/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
	pub fn post_object<B, T>(&self, url: ::reqwest::Url, body: &B) -> ::Result<T>
		where B: ::serde::Serialize, T: ::serde::Deserialize {

		let request = self.client.post(url).header(USER_AGENT.clone()).header(APPLICATION_JSON.clone()).form(body);
		let response = send(request)?;
		let object = json(response)?;
		Ok(object)
	}
}

lazy_static! {
	static ref USER_AGENT: ::reqwest::header::UserAgent = ::reqwest::header::UserAgent(format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
	static ref APPLICATION_JSON: ::reqwest::header::Accept = ::reqwest::header::Accept::json();
	static ref APPLICATION_ZIP: ::reqwest::header::Accept = ::reqwest::header::Accept(vec![::reqwest::header::qitem(mime!(Application/("zip")))]);
}

/// A login failure response.
#[derive(Debug, Deserialize)]
struct LoginFailureResponse {
	message: String,
}

fn send(request: ::reqwest::RequestBuilder) -> ::Result<::reqwest::Response> {
	let response = request.send()?;
	Ok(match *response.status() {
		::reqwest::StatusCode::Ok =>
			response,

		::reqwest::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = json(response)?;
			bail!(::ErrorKind::LoginFailure(object.message))
		},

		::reqwest::StatusCode::Found =>
			bail!(::ErrorKind::LoginFailure("Redirected to login page.".to_string())),

		code =>
			bail!(::ErrorKind::StatusCode(code)),
	})
}

fn json<T>(mut response: ::reqwest::Response) -> ::Result<T> where T: ::serde::Deserialize {
	match response.headers().get() {
		Some(&::reqwest::header::ContentType(::mime::Mime(::mime::TopLevel::Application, ::mime::SubLevel::Json, _))) =>
			(),
		Some(&::reqwest::header::ContentType(ref mime)) =>
			bail!(::ErrorKind::MalformedResponse(format!("Unexpected Content-Type header: {}", mime))),
		None =>
			bail!(::ErrorKind::MalformedResponse("No Content-Type header".to_string())),
	}

	let object = response.json()?;

	Ok(object)
}
