/// Entry-point to the https://mods.factorio.com API
#[derive(Debug)]
pub struct API {
	base_url: ::reqwest::Url,
	login_url: ::reqwest::Url,
	mods_url: ::reqwest::Url,
	client: ::reqwest::Client,
}

impl API {
	/// Constructs an API object with the given parameters.
	pub fn new(base_url: Option<&str>, login_url: Option<&str>, client: Option<::reqwest::Client>) -> ::Result<API> {
		let base_url = match base_url {
			Some(base_url) => ::reqwest::Url::parse(base_url)?,
			None => BASE_URL.clone(),
		};

		let login_url = match login_url {
			Some(login_url) => ::reqwest::Url::parse(login_url)?,
			None => LOGIN_URL.clone(),
		};

		let mods_url = base_url.join("/api/mods")?;
		if mods_url.cannot_be_a_base() {
			bail!("URL {} cannot be a base.", mods_url);
		}

		let mut client = match client {
			Some(client) => client,
			None => ::reqwest::Client::new()?,
		};

		let base_url_host = base_url.host_str().ok_or_else(|| format!("URL {} does not have a hostname.", base_url))?.to_string();
		client.redirect(::reqwest::RedirectPolicy::custom(move |url, _| {
			if let Some(host) = url.host_str() {
				if host != base_url_host {
					return Ok(true);
				}
			}

			Ok(url.path() != "/login")
		}));

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
	pub fn get(&self, mod_name: &::factorio_mods_common::ModName) -> ::Result<::Mod> {
		let mut mods_url = self.mods_url.clone();
		mods_url.path_segments_mut().unwrap().push(mod_name);
		::util::get_object(&self.client, mods_url)
	}

	/// Logs in to the web API using the given username and password and returns a credentials object.
	pub fn login(&self, username: ::factorio_mods_common::ServiceUsername, password: &str) -> ::Result<::factorio_mods_common::UserCredentials> {
		let token = {
			let response: [::factorio_mods_common::ServiceToken; 1] =
				::util::post_object(&self.client, self.login_url.clone(), &[("username", &*username), ("password", password)])?;
			response[0].clone()
		};
		Ok(::factorio_mods_common::UserCredentials::new(username, token))
	}

	/// Downloads the file for the specified mod release and returns a reader to the file contents.
	pub fn download(
		&self,
		release: &::ModRelease,
		user_credentials: &::factorio_mods_common::UserCredentials,
	) -> ::Result<impl ::std::io::Read> {
		let mut download_url = self.base_url.join(release.download_url())?;
		download_url.query_pairs_mut()
			.append_pair("username", user_credentials.username())
			.append_pair("token", user_credentials.token());

		let response = ::util::get(&self.client, download_url)?;

		let file_size = {
			let headers = response.headers();

			match headers.get() {
				Some(&::reqwest::header::ContentType(::mime::Mime(::mime::TopLevel::Application, ::mime::SubLevel::Ext(ref sublevel), _))) if sublevel == "zip" =>
					(),
				Some(&::reqwest::header::ContentType(ref mime)) =>
					bail!(::ErrorKind::MalformedResponse(format!("Unexpected Content-Type header: {}", mime))),
				None =>
					bail!(::ErrorKind::MalformedResponse("No Content-Type header".to_string())),
			}

			if let Some(&::reqwest::header::ContentLength(ref file_size)) = headers.get() {
				*file_size
			}
			else {
				bail!(::ErrorKind::MalformedResponse("No Content-Length header".to_string()));
			}
		};

		let expected_file_size = **release.file_size();
		if file_size != expected_file_size {
			bail!(::ErrorKind::MalformedResponse(format!("Mod file has incorrect size {} bytes, expected {} bytes.", file_size, expected_file_size)));
		}

		Ok(response)
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

const DEFAULT_ORDER: SearchOrder = SearchOrder::MostDownloaded;
lazy_static! {
	static ref BASE_URL: ::reqwest::Url = ::reqwest::Url::parse("https://mods.factorio.com/").unwrap();
	static ref LOGIN_URL: ::reqwest::Url = ::reqwest::Url::parse("https://auth.factorio.com/api-login").unwrap();
	static ref DEFAULT_PAGE_SIZE: ::ResponseNumber = ::ResponseNumber::new(25);
}


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
		assert_eq!(&**mod_.title(), "Bob's Functions Library mod");
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

		let mod_ = api.get(&::factorio_mods_common::ModName::new("boblibrary".to_string())).unwrap();
		println!("{:?}", mod_);
		assert_eq!(&**mod_.title(), "Bob's Functions Library mod");
	}
}
