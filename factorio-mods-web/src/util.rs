/// GETs the given URL using the given client, and returns the raw response.
pub fn get(client: &::hyper::Client, url: ::hyper::Url) -> ::Result<::hyper::client::Response> {
	let response = client.get(url).send()?;
	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),

		::hyper::status::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = ::serde_json::from_reader(response)?;
			Err(::ErrorKind::LoginFailure(object.message).into())
		},

		::hyper::status::StatusCode::Found => {
			Err(::ErrorKind::LoginFailure("Redirected to login page.".to_string()).into())
		},

		code => Err(::ErrorKind::StatusCode(code).into()),
	}
}

/// GETs the given URL using the given client, and deserializes the response as a JSON object.
pub fn get_object<T>(client: &::hyper::Client, url: ::hyper::Url) -> ::Result<T> where T: ::serde::Deserialize {
	let response = get(client, url)?;
	let object = ::serde_json::from_reader(response)?;
	Ok(object)
}

/// POSTs the given URL using the given client and request body, and returns the raw response.
pub fn post(client: &::hyper::Client, url: ::hyper::Url, body: String) -> ::Result<::hyper::client::Response> {
	let response =
		client.post(url)
		.header(CONTENT_TYPE_URLENCODED.clone())
		.body(&body)
		.send()?;

	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),

		::hyper::status::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = ::serde_json::from_reader(response)?;
			Err(::ErrorKind::LoginFailure(object.message).into())
		},

		::hyper::status::StatusCode::Found => {
			Err(::ErrorKind::LoginFailure("Redirected to login page.".to_string()).into())
		},

		code => Err(::ErrorKind::StatusCode(code).into()),
	}
}

/// POSTs the given URL using the given client and request body, and deserializes the response as a JSON object.
pub fn post_object<T>(client: &::hyper::Client, url: ::hyper::Url, body: String) -> ::Result<T> where T: ::serde::Deserialize {
	let response = post(client, url, body)?;
	let object = ::serde_json::from_reader(response)?;
	Ok(object)
}

lazy_static! {
	static ref CONTENT_TYPE_URLENCODED: ::hyper::header::ContentType =
		::hyper::header::ContentType(
			::hyper::mime::Mime(::hyper::mime::TopLevel::Application, ::hyper::mime::SubLevel::WwwFormUrlEncoded, vec![]));
}

/// A login failure response.
#[derive(Debug, Deserialize)]
struct LoginFailureResponse {
	message: String,
}
