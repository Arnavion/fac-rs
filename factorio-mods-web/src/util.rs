/// GETs the given URL using the given client, and returns the raw response.
pub fn get(client: &::reqwest::Client, url: ::reqwest::Url) -> ::Result<::reqwest::Response> {
	let response = client.get(url).send()?;
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

/// GETs the given URL using the given client, and deserializes the response as a JSON object.
pub fn get_object<T>(client: &::reqwest::Client, url: ::reqwest::Url) -> ::Result<T> where T: ::serde::Deserialize {
	let response = get(client, url)?;
	let object = json(response)?;
	Ok(object)
}

/// POSTs the given URL using the given client and request body, and returns the raw response.
pub fn post<B>(client: &::reqwest::Client, url: ::reqwest::Url, body: &B) -> ::Result<::reqwest::Response> where B: ::serde::Serialize {
	let response =
		client.post(url)
		.form(body)
		.send()?;

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

/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
pub fn post_object<B, T>(client: &::reqwest::Client, url: ::reqwest::Url, body: &B) -> ::Result<T>
	where B: ::serde::Serialize, T: ::serde::Deserialize {
	let response = post(client, url, body)?;
	let object = json(response)?;
	Ok(object)
}

/// A login failure response.
#[derive(Debug, Deserialize)]
struct LoginFailureResponse {
	message: String,
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
