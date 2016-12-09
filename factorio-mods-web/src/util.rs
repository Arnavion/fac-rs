/// GETs the given URL using the given client, and returns the raw response.
pub fn get(client: &::hyper::Client, url: ::hyper::Url) -> ::Result<::hyper::client::Response> {
	let response = client.get(url).send()?;
	Ok(match response.status {
		::hyper::status::StatusCode::Ok =>
			response,

		::hyper::status::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = json(response)?;
			bail!(::ErrorKind::LoginFailure(object.message))
		},

		::hyper::status::StatusCode::Found =>
			bail!(::ErrorKind::LoginFailure("Redirected to login page.".to_string())),

		code =>
			bail!(::ErrorKind::StatusCode(code)),
	})
}

/// GETs the given URL using the given client, and deserializes the response as a JSON object.
pub fn get_object<T>(client: &::hyper::Client, url: ::hyper::Url) -> ::Result<T> where T: ::serde::Deserialize {
	let response = get(client, url)?;
	let object = json(response)?;
	Ok(object)
}

/// POSTs the given URL using the given client and request body, and returns the raw response.
pub fn post(client: &::hyper::Client, url: ::hyper::Url, body: String) -> ::Result<::hyper::client::Response> {
	let response =
		client.post(url)
		.header(::hyper::header::ContentType::form_url_encoded())
		.body(&body)
		.send()?;

	Ok(match response.status {
		::hyper::status::StatusCode::Ok =>
			response,

		::hyper::status::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = json(response)?;
			bail!(::ErrorKind::LoginFailure(object.message))
		},

		::hyper::status::StatusCode::Found =>
			bail!(::ErrorKind::LoginFailure("Redirected to login page.".to_string())),

		code =>
			bail!(::ErrorKind::StatusCode(code)),
	})
}

/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
pub fn post_object<T>(client: &::hyper::Client, url: ::hyper::Url, body: String) -> ::Result<T> where T: ::serde::Deserialize {
	let response = post(client, url, body)?;
	let object = json(response)?;
	Ok(object)
}

/// A login failure response.
#[derive(Debug, Deserialize)]
struct LoginFailureResponse {
	message: String,
}

fn json<T>(response: ::hyper::client::response::Response) -> ::Result<T> where T: ::serde::Deserialize {
	match (&response.headers).get() {
		Some(&::hyper::header::ContentType(::hyper::mime::Mime(::hyper::mime::TopLevel::Application, ::hyper::mime::SubLevel::Json, _))) =>
			(),
		Some(&::hyper::header::ContentType(ref mime)) =>
			bail!(::ErrorKind::MalformedJsonResponse(format!("Unexpected Content-Type header: {}", mime))),
		None =>
			bail!(::ErrorKind::MalformedJsonResponse("No Content-Type header".to_string())),
	}

	let object = ::serde_json::from_reader(response)?;

	Ok(object)
}
