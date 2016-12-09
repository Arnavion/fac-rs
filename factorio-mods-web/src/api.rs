/// Entry-point to the https://mods.factorio.com API
#[derive(Debug)]
pub struct API {
	base_url: ::hyper::Url,
	login_url: ::hyper::Url,
	mods_url: ::hyper::Url,
	client: ::hyper::Client,
}

impl API {
	/// Constructs an API object with the given parameters.
	pub fn new(base_url: Option<&str>, login_url: Option<&str>, client: Option<::hyper::Client>) -> ::Result<API> {
		let base_url = base_url.unwrap_or_else(|| BASE_URL);
		let base_url = ::hyper::Url::parse(base_url)?;

		let login_url = login_url.unwrap_or_else(|| LOGIN_URL);
		let login_url = ::hyper::Url::parse(login_url)?;

		let mods_url = base_url.join("/api/mods")?;
		if mods_url.cannot_be_a_base() {
			bail!("URL {} cannot be a base.", mods_url);
		}

		let mut client = client.unwrap_or_else(::hyper::Client::new);
		client.set_redirect_policy(::hyper::client::RedirectPolicy::FollowIf(should_follow_redirect));

		Ok(API {
			base_url: base_url,
			login_url: login_url,
			mods_url: mods_url,
			client: client,
		})
	}

	/// Searches for mods matching the given criteria.
	pub fn search<'a>(
		&'a self,
		query: &str,
		tags: &[&::TagName],
		order: Option<&SearchOrder>,
		page_size: Option<&::ResponseNumber>,
		page: Option<::PageNumber>
	) -> impl Iterator<Item = ::Result<::SearchResponseMod>> + 'a {
		let tags_query = ::itertools::join(tags, ",");
		let order = order.unwrap_or(&DEFAULT_ORDER).to_query_parameter();
		let page_size = (page_size.unwrap_or(&DEFAULT_PAGE_SIZE)).to_string();
		let page = page.unwrap_or_else(|| ::PageNumber::new(1));

		let mut mods_url = self.mods_url.clone();
		mods_url.query_pairs_mut()
			.append_pair("q", query)
			.append_pair("tags", &tags_query)
			.append_pair("order", order)
			.append_pair("page_size", &page_size);

		::search::search(&self.client, mods_url, page)
	}

	/// Gets information about the specified mod.
	pub fn get(&self, mod_name: ::factorio_mods_common::ModName) -> ::Result<::Mod> {
		let mut mods_url = self.mods_url.clone();
		mods_url.path_segments_mut().unwrap().push(&mod_name);
		::util::get_object(&self.client, mods_url)
	}

	/// Logs in to the web API using the given username and password and returns a credentials object.
	pub fn login(&self, username: ::factorio_mods_common::ServiceUsername, password: &str) -> ::Result<::factorio_mods_common::UserCredentials> {
		let body =
			::url::form_urlencoded::Serializer::new(String::new())
			.append_pair("username", &username)
			.append_pair("password", password)
			.finish();
		let response: LoginSuccessResponse = ::util::post_object(&self.client, self.login_url.clone(), body)?;
		let token = response.0.into_iter().next().ok_or("Malformed login response")?;
		Ok(::factorio_mods_common::UserCredentials::new(username, token))
	}

	/// Downloads the file for the specified mod release and returns a reader to the file contents.
	pub fn download(
		&self,
		release: &::ModRelease,
		user_credentials: &::factorio_mods_common::UserCredentials,
	) -> ::Result<::std::io::BufReader<::hyper::client::Response>> {
		let mut download_url = self.base_url.join(release.download_url())?;
		download_url.query_pairs_mut()
			.append_pair("username", user_credentials.username())
			.append_pair("token", user_credentials.token());

		let response = ::util::get(&self.client, download_url)?;

		let file_size = {
			let headers = &response.headers;

			let mime =
				if let Some(&::hyper::header::ContentType(ref mime)) = headers.get() {
					mime
				}
				else {
					bail!(::ErrorKind::MalformedModDownloadResponse("No Content-Type header".to_string()));
				};

			if mime != &*APPLICATION_ZIP {
				bail!(::ErrorKind::MalformedModDownloadResponse(format!("Unexpected Content-Type header: {}", mime)));
			}

			if let Some(&::hyper::header::ContentLength(ref file_size)) = headers.get() {
				*file_size
			}
			else {
				bail!(::ErrorKind::MalformedModDownloadResponse("No Content-Length header".to_string()));
			}
		};

		if file_size != **release.file_size() {
			bail!(::ErrorKind::MalformedModDownloadResponse(format!("Downloaded file has incorrect size ({}), expected {}.", file_size, release.file_size())));
		}

		Ok(::std::io::BufReader::new(response))
	}
}

/// Search order
pub enum SearchOrder {
	/// A to Z
	Alphabetically,

	/// Most to least
	MostDownloaded,

	/// Newest to oldest
	RecentlyUpdated,
}

impl SearchOrder {
	/// Converts the SearchOrder to a string that can be ised in the search URL's querystring
	fn to_query_parameter(&self) -> &'static str {
		match *self {
			SearchOrder::Alphabetically => "alpha",
			SearchOrder::MostDownloaded => "top",
			SearchOrder::RecentlyUpdated => "updated",
		}
	}
}

const BASE_URL: &'static str = "https://mods.factorio.com/";
const LOGIN_URL: &'static str = "https://auth.factorio.com/api-login";
const DEFAULT_ORDER: SearchOrder = SearchOrder::MostDownloaded;
lazy_static! {
	static ref DEFAULT_PAGE_SIZE: ::ResponseNumber = ::ResponseNumber::new(25);
	static ref APPLICATION_ZIP: ::hyper::mime::Mime =
		::hyper::mime::Mime(::hyper::mime::TopLevel::Application, ::hyper::mime::SubLevel::Ext("zip".to_string()), vec![]);
}

fn should_follow_redirect(url: &::hyper::Url) -> bool {
	if let Some(host) = url.host_str() {
		if host != "mods.factorio.com" {
			return true;
		}
	}

	url.path() != "/login"
}

#[derive(Clone, Debug, Deserialize, newtype_ref)]
struct LoginSuccessResponse(Vec<::factorio_mods_common::ServiceToken>);


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn search_list_all_mods() {
		let api = API::new(None, None, None).unwrap();

		let iter = api.search("", &[], None, None, None);
		let mods = iter.map(|m| m.unwrap()); // Ensure all are Ok()
		let count = mods.count();
		println!("Found {} mods", count);
		assert!(count > 500); // 700+ as of 2016-10-03
	}

	#[test]
	fn search_by_title() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("bob's functions library mod", &[], None, None, None);
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		assert!(&**mod_.title() == "Bob's Functions Library mod");
	}

	#[test]
	fn search_by_tag() {
		let api = API::new(None, None, None).unwrap();

		let mut iter = api.search("", &vec![&::TagName::new("logistics".to_string())], None, None, None);
		let mod_ = iter.next().unwrap().unwrap();
		println!("{:?}", mod_);
		let mut tags = mod_.tags().iter().filter(|tag| &**tag.name() == "logistics");
		let tag = tags.next().unwrap();
		println!("{:?}", tag);
	}

	#[test]
	fn get() {
		let api = API::new(None, None, None).unwrap();

		let mod_ = api.get(::factorio_mods_common::ModName::new("boblibrary".to_string())).unwrap();
		println!("{:?}", mod_);
		assert!(&**mod_.title() == "Bob's Functions Library mod");
	}
}
