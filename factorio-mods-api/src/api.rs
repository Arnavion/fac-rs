make_newtype!(pub PageNumber(u64));

make_deserializable!(pub struct UserCredentials {
	username: String,
	token: String,
});

#[derive(Debug)]
pub struct API {
	base_url: ::hyper::Url,
	login_url: ::hyper::Url,
	mods_url: ::hyper::Url,
	client: ::hyper::Client,
}

impl API {
	pub fn new(base_url: Option<&str>, login_url: Option<&str>, client: Option<::hyper::Client>) -> ::error::Result<API> {
		let base_url = base_url.unwrap_or_else(|| BASE_URL);
		let base_url = try!(::hyper::Url::parse(base_url));

		let login_url = login_url.unwrap_or_else(|| LOGIN_URL);
		let login_url = try!(::hyper::Url::parse(login_url));

		let mods_url = try!(base_url.join("/api/mods"));
		if mods_url.cannot_be_a_base() {
			return Err(format!("URL {} cannot be a base.", mods_url).into());
		}

		let client = client.unwrap_or_else(::hyper::Client::new);

		Ok(API {
			base_url: base_url,
			login_url: login_url,
			mods_url: mods_url,
			client: client,
		})
	}

	pub fn search<'a>(
		&'a self,
		query: &str,
		tags: &Vec<&::factorio_mods_common::TagName>,
		order: Option<String>,
		page_size: Option<PageNumber>,
		page: Option<PageNumber>
	) -> ::error::Result<SearchResultsIterator<'a>> {
		let tags_query = ::itertools::join(tags, ",");
		let order = order.unwrap_or_else(|| DEFAULT_ORDER.to_string());
		let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).0.to_string();
		let page = page.unwrap_or_else(|| PageNumber(1));

		let mut mods_url = self.mods_url.clone();
		mods_url.query_pairs_mut()
			.append_pair("q", query)
			.append_pair("tags", &tags_query)
			.append_pair("order", &order)
			.append_pair("page_size", &page_size);

		Ok(SearchResultsIterator {
			client: &self.client,
			mods_url: mods_url,
			current_page_number: page,
			current_page: None,
			errored: false,
		})
	}

	pub fn get(&self, mod_name: ::factorio_mods_common::ModName) -> ::error::Result<::factorio_mods_common::Mod> {
		let mut mods_url = self.mods_url.clone();
		mods_url.path_segments_mut().unwrap().push(&mod_name);
		get_object(&self.client, mods_url)
	}

	pub fn login(&self, username: String, password: &str) -> ::error::Result<UserCredentials> {
		let mut serializer = ::url::form_urlencoded::Serializer::new(String::new());
		serializer.append_pair("username", &username).append_pair("password", password);
		let response: LoginSuccessResponse = try!(post_object(&self.client, self.login_url.clone(), serializer));
		let mut token_list = response.0;
		let token = try!(token_list.drain(..).next().ok_or_else(|| "Malformed login response"));
		Ok(UserCredentials { username: username, token: token })
	}
}

const BASE_URL: &'static str = "https://mods.factorio.com/";
const LOGIN_URL: &'static str = "https://auth.factorio.com/api-login";
const DEFAULT_PAGE_SIZE: PageNumber = PageNumber(25);
const DEFAULT_ORDER: &'static str = "top";
lazy_static! {
	static ref CONTENT_TYPE_URLENCODED: ::hyper::header::ContentType = ::hyper::header::ContentType("application/x-www-form-urlencoded".parse().unwrap());
}

fn get(client: &::hyper::Client, url: ::hyper::Url) -> ::error::Result<::hyper::client::Response> {
	let response = try!(client.get(url).send());
	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),
		code => Err(::error::ErrorKind::StatusCode(code).into())
	}
}

fn get_object<T>(client: &::hyper::Client, url: ::hyper::Url) -> ::error::Result<T> where T: ::serde::Deserialize {
	let response = try!(get(client, url));
	let object = try!(::serde_json::from_reader(response));
	return Ok(object);
}

fn post(client: &::hyper::Client, url: ::hyper::Url, body: ::url::form_urlencoded::Serializer<String>) -> ::error::Result<::hyper::client::Response> {
	let mut body = body;
	let body = body.finish();

	let response = try!(
		client.post(url)
		.header(CONTENT_TYPE_URLENCODED.clone())
		.body(&body)
		.send());

	match response.status {
		::hyper::status::StatusCode::Ok => Ok(response),

		::hyper::status::StatusCode::Unauthorized => {
			let object: LoginFailureResponse = try!(::serde_json::from_reader(response));
			Err(::error::ErrorKind::LoginFailure(object.message).into())
		},

		code => Err(::error::ErrorKind::StatusCode(code).into())
	}
}

fn post_object<T>(client: &::hyper::Client, url: ::hyper::Url, body: ::url::form_urlencoded::Serializer<String>) -> ::error::Result<T> where T: ::serde::Deserialize {
	let response = try!(post(client, url, body));
	let object = try!(::serde_json::from_reader(response));
	return Ok(object);
}

make_deserializable!(struct ResponseNumber(u64));

make_deserializable!(struct SearchResponse {
	pagination: SearchResponsePagination,
	results: Vec<SearchResponseMod>,
});

make_deserializable!(struct SearchResponsePagination {
	page_count: PageNumber,
	page: PageNumber,

	page_size: ResponseNumber,
	count: ResponseNumber,

	links: SearchResponsePaginationLinks,
});

make_deserializable!(struct SearchResponsePaginationLinks {
	prev: Option<String>,
	next: Option<String>,
	first: Option<String>,
	last: Option<String>,
});

make_deserializable!(pub struct SearchResponseMod {
	pub id: ::factorio_mods_common::ModId,

	pub name: ::factorio_mods_common::ModName,
	pub owner: ::factorio_mods_common::AuthorNames,
	pub title: ::factorio_mods_common::ModTitle,
	pub summary: ::factorio_mods_common::ModSummary,

	pub github_path: ::factorio_mods_common::Url,
	pub homepage: ::factorio_mods_common::Url,
	pub license_name: ::factorio_mods_common::LicenseName,
	pub license_url: ::factorio_mods_common::Url,

	pub game_versions: Vec<::factorio_mods_common::GameVersion>,

	pub created_at: ::factorio_mods_common::DateTime,
	pub updated_at: ::factorio_mods_common::DateTime,
	pub latest_release: ::factorio_mods_common::ModRelease,

	pub current_user_rating: Option<::serde_json::Value>,
	pub downloads_count: ::factorio_mods_common::DownloadCount,
	pub visits_count: ::factorio_mods_common::VisitCount,
	pub tags: ::factorio_mods_common::Tags,
});

#[derive(Debug)]
pub struct SearchResultsIterator<'a> {
	client: &'a ::hyper::Client,
	mods_url: ::hyper::Url,
	current_page_number: PageNumber,
	current_page: Option<SearchResponse>,
	errored: bool,
}

impl<'a> Iterator for SearchResultsIterator<'a> {
	type Item = ::error::Result<SearchResponseMod>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.errored {
			return None;
		}

		match self.current_page {
			Some(ref mut page) if !page.results.is_empty() => {
				let result = page.results.remove(0);
				return Some(Ok(result));
			}

			Some(_) => {
				self.current_page_number.0 += 1;
				self.current_page = None;
				return self.next()
			},

			None => {
				let mut mods_url = self.mods_url.clone();

				mods_url.query_pairs_mut().append_pair("page", &self.current_page_number.to_string());

				match get_object(self.client, mods_url) {
					Ok(page) => {
						self.current_page = Some(page);
						return self.next();
					},

					Err(err) => match err.kind() {
						&::error::ErrorKind::StatusCode(::hyper::status::StatusCode::NotFound) => {
							self.errored = true;
							return None;
						}

						_ => {
							self.errored = true;
							return Some(Err(err));
						}
					},
				}
			}
		}
	}
}

make_newtype!(LoginSuccessResponse(Vec<String>));

make_deserializable!(struct LoginFailureResponse {
	message: String,
});


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn search_list_all_mods() {
		let api = API::new(None, None, None).unwrap();

		let iter = api.search("", &vec![], None, None, None).unwrap();
		let mods = iter.map(|m| m.unwrap()); // Ensure all are Ok()
		let count = mods.count();
		println!("Found {} mods", count);
		assert!(count > 500); // 700+ as of 2016-10-03
	}

	#[test]
	fn search_by_title() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("bob's functions library mod", &vec![], None, None, None).unwrap();
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		assert!(mod_.title.0 == "Bob's Functions Library mod");
	}

	#[test]
	fn search_by_tag() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("", &vec![&::factorio_mods_common::TagName("logistics".to_string())], None, None, None).unwrap();
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		let mut tags = mod_.tags.0.iter().filter(|tag| tag.name.0 == "logistics");
		let tag = tags.next().unwrap();
		println!("{:?}", tag);
	}

	#[test]
	fn get() {
		let api = API::new(None, None, None).unwrap();

		let mod_ = api.get(::factorio_mods_common::ModName("boblibrary".to_string())).unwrap();
		println!("{:?}", mod_);
		assert!(mod_.title.0 == "Bob's Functions Library mod");
	}
}
