/// Entry-point to the <https://mods.factorio.com/> API
#[derive(Debug)]
pub struct API {
	base_url: reqwest::Url,
	mods_url: reqwest::Url,
	login_url: reqwest::Url,
	client: crate::client::Client,
}

impl API {
	/// Constructs an API object with the given parameters.
	pub fn new(builder: Option<reqwest::ClientBuilder>) -> Result<Self, crate::Error> {
		static BASE_URL: once_cell::sync::Lazy<reqwest::Url> =
			once_cell::sync::Lazy::new(|| "https://mods.factorio.com/".parse().unwrap());
		static MODS_URL: once_cell::sync::Lazy<reqwest::Url> =
			once_cell::sync::Lazy::new(|| "https://mods.factorio.com/api/mods?page_size=max".parse().unwrap());
		static LOGIN_URL: once_cell::sync::Lazy<reqwest::Url> =
			once_cell::sync::Lazy::new(|| "https://auth.factorio.com/api-login".parse().unwrap());

		Ok(API {
			base_url: BASE_URL.clone(),
			mods_url: MODS_URL.clone(),
			login_url: LOGIN_URL.clone(),
			client: crate::client::Client::new(builder)?,
		})
	}

	/// Searches for mods matching the given criteria.
	pub fn search<'a>(&'a self, query: &str) -> impl futures_core::Stream<Item = Result<crate::SearchResponseMod, crate::Error>> + 'a {
		let query = query.to_lowercase();

		let mut next_page_url = self.mods_url.clone();

		Box::pin(async_stream::try_stream! {
			loop {
				let next_page: Result<(PagedResponse<crate::SearchResponseMod>, _), _> = self.client.get_object(next_page_url).await;
				match next_page {
					Ok((page, _)) => {
						for mod_ in page.results {
							if
								mod_.name.0.to_lowercase().contains(&query) ||
								mod_.title.0.to_lowercase().contains(&query) ||
								mod_.owner.iter().any(|owner| owner.0.to_lowercase().contains(&query)) ||
								mod_.summary.0.to_lowercase().contains(&query)
							{
								yield mod_;
							}
						}

						if let Some(url) = page.pagination.and_then(|pagination| pagination.links.next) {
							next_page_url = url;
						}
						else {
							return;
						}
					},

					Err(crate::Error::StatusCode(_, reqwest::StatusCode::NOT_FOUND)) => return,

					Err(err) => {
						Err(err)?;
						return;
					},
				}
			}
		})
	}

	/// Gets information about the specified mod.
	pub fn get(&self, mod_name: &factorio_mods_common::ModName) -> impl std::future::Future<Output = Result<crate::Mod, crate::Error>> {
		let mut mod_url = self.mods_url.clone();
		mod_url.path_segments_mut().unwrap().push(&mod_name.0);
		let future = self.client.get_object(mod_url);

		async {
			let (mod_, _) = future.await?;
			Ok(mod_)
		}
	}

	/// Logs in to the web API using the given username and password and returns a credentials object.
	pub fn login(
		&self,
		username: factorio_mods_common::ServiceUsername,
		password: &str,
	) -> impl std::future::Future<Output = Result<factorio_mods_common::UserCredentials, crate::Error>> {
		let future = self.client.post_object(self.login_url.clone(), &[("username", &*username.0), ("password", password)]);

		async {
			let ((token,), _) = future.await?;
			Ok(factorio_mods_common::UserCredentials { username, token })
		}
	}

	/// Get the filesize for the specified mod release.
	pub fn get_filesize(
		&self,
		release: &crate::ModRelease,
		user_credentials: &factorio_mods_common::UserCredentials,
	) -> impl std::future::Future<Output = Result<u64, crate::Error>> {
		let download_url = match self.base_url.join(&release.download_url.0) {
			Ok(mut download_url) => {
				download_url.query_pairs_mut()
					.append_pair("username", &user_credentials.username.0)
					.append_pair("token", &user_credentials.token.0);

				download_url
			},

			Err(err) =>
				return futures_util::future::Either::Left(futures_util::future::ready(Err(
					crate::Error::Parse(format!("{}/{}", self.base_url, release.download_url), err)))),
		};

		let head = self.client.head_zip(download_url);

		futures_util::future::Either::Right(async {
			let (response, download_url) = head.await?;
			let len = match response.headers().get(reqwest::header::CONTENT_LENGTH) {
				Some(len) => len,
				None => return Err(crate::Error::MalformedResponse(download_url, "No Content-Length header".to_owned())),
			};
			let len = match len.to_str() {
				Ok(len) => len,
				Err(err) => return Err(crate::Error::MalformedResponse(download_url, format!("Malformed Content-Length header: {}", err))),
			};
			let len = match len.parse() {
				Ok(len) => len,
				Err(err) => return Err(crate::Error::MalformedResponse(download_url, format!("Malformed Content-Length header: {}", err))),
			};
			Ok(len)
		})
	}

	/// Downloads the file for the specified mod release and returns a reader to the file contents.
	pub fn download(
		&self,
		release: &crate::ModRelease,
		user_credentials: &factorio_mods_common::UserCredentials,
		range: Option<&str>,
	) -> impl futures_core::Stream<Item = Result<bytes::Bytes, crate::Error>> {
		let download_url = match self.base_url.join(&release.download_url.0) {
			Ok(mut download_url) => {
				download_url.query_pairs_mut()
					.append_pair("username", &user_credentials.username.0)
					.append_pair("token", &user_credentials.token.0);

				download_url
			},

			Err(err) =>
				return futures_util::future::Either::Left(futures_util::stream::once(futures_util::future::ready(Err(
					crate::Error::Parse(format!("{}/{}", self.base_url, release.download_url), err))))),
		};

		let fetch = self.client.get_zip(download_url, range);

		futures_util::future::Either::Right(async_stream::try_stream! {
			let (response, download_url) = fetch.await?;
			let mut response = response.bytes_stream();
			let mut download_url = Some(download_url);

			loop {
				let chunk = futures_util::TryStreamExt::try_next(&mut response).await;
				match chunk {
					Ok(Some(chunk)) => yield chunk,

					Ok(None) => return,

					Err(err) => {
						if let Some(download_url) = download_url.take() {
							Err(crate::Error::Http(download_url, err))?;
						}

						return;
					},
				}
			}
		})
	}
}

/// A single page of a paged response.
#[derive(Debug, serde::Deserialize)]
struct PagedResponse<T> {
	pagination: Option<Pagination>,
	results: Vec<T>,
}

/// Pagination information in a paged response.
#[derive(Debug, serde::Deserialize)]
struct Pagination {
	links: PaginationLinks,
}

/// Pagination link information in a paged response.
#[derive(Debug, serde::Deserialize)]
struct PaginationLinks {
	next: Option<reqwest::Url>,
}

#[cfg(test)]
mod tests {
	#[tokio::test]
	async fn search_list_all_mods() {
		use futures_util::TryStreamExt;

		let api = super::API::new(None).unwrap();
		let count =
			api.search("")
			.try_fold(0_usize, |count, _| futures_util::future::ready(Ok(count + 1)))
			.await.unwrap();
		println!("Found {} mods", count);
		assert!(count > 5200); // 5200+ as of 2019-12-14
	}

	#[tokio::test]
	async fn search_by_title() {
		let api = super::API::new(None).unwrap();

		let mut search_results = api.search("bob's functions library mod");
		while let Some(mod_) = futures_util::StreamExt::next(&mut search_results).await {
			println!("{:?}", mod_);
			let mod_ = mod_.unwrap();
			if mod_.title.0 == "Bob's Functions Library mod" {
				return;
			}
		}

		panic!("boblibrary not found");
	}

	#[tokio::test]
	async fn search_non_existing() {
		let api = super::API::new(None).unwrap();
		let mut search_results = api.search("arnavion's awesome mod");
		assert!(futures_util::StreamExt::next(&mut search_results).await.is_none());
	}

	#[tokio::test]
	async fn get() {
		let api = super::API::new(None).unwrap();

		let mod_name = factorio_mods_common::ModName("boblibrary".to_owned());
		let mod_ = api.get(&mod_name).await.unwrap();
		println!("{:?}", mod_);
		assert_eq!(mod_.title.0, "Bob's Functions Library mod");
	}
}
