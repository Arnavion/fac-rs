#![allow(
	clippy::cast_possible_truncation,
	clippy::cast_sign_loss,
)]

pub(super) struct WebReader<'a> {
	api: &'a factorio_mods_web::Api,
	release: std::rc::Rc<factorio_mods_web::ModRelease>,
	user_credentials: std::rc::Rc<factorio_mods_common::UserCredentials>,

	len: u64,
	pos: u64,

	// Up to three regions might be in use at any time:
	// - A: The region that contains the start of the current read.
	// - B: The region that contains the end of the current read. Same as A unless the read crosses a region boundary.
	//      Regions are larger than the largest expected read, so a read will never cross *two* boundaries.
	// - C: One region past B, if it exists, to hold the reusable ReqwestResponseReader.
	content_cache: uluru::LRUCache<(u64, DataRegion<'a>), 3>,
}

enum DataRegion<'a> {
	Download(std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<(ReqwestResponseReader<'a>, Vec<u8>)>> + 'a>>),
	Downloaded(Vec<u8>),
}

type ReqwestResponseReader<'a> = futures_util::stream::IntoAsyncRead<std::pin::Pin<Box<dyn futures_core::Stream<Item = std::io::Result<bytes::Bytes>> + 'a>>>;

const REGION_LEN_MAX: usize = 1024 * 8;

impl<'a> WebReader<'a> {
	pub(super) async fn new(
		api: &'a factorio_mods_web::Api,
		release: std::rc::Rc<factorio_mods_web::ModRelease>,
		user_credentials: std::rc::Rc<factorio_mods_common::UserCredentials>,
	) -> Result<Self, factorio_mods_web::Error> {
		let len = api.get_filesize(&release, &user_credentials).await?;
		Ok(WebReader {
			api,
			release,
			user_credentials,

			len,
			pos: 0,

			content_cache: Default::default(),
		})
	}
}

impl futures_util::io::AsyncRead for WebReader<'_> {
	fn poll_read(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut [u8]) -> std::task::Poll<std::io::Result<usize>> {
		let this = &mut *self;

		if this.pos >= this.len {
			return std::task::Poll::Ready(Ok(0));
		}

		let (key, offset) = (this.pos / REGION_LEN_MAX as u64, (this.pos % REGION_LEN_MAX as u64) as usize);

		let content = loop {
			if let Some((_, region)) = this.content_cache.find(|entry| entry.0 == key) {
				match region {
					DataRegion::Download(download) => match download.as_mut().poll(cx) {
						std::task::Poll::Ready(download) => {
							let (reader, content) =
								download.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

							*region = DataRegion::Downloaded(content);

							if (key + 1).saturating_mul(REGION_LEN_MAX as u64) < this.len && this.content_cache.find(|entry| entry.0 == key + 1).is_none() {
								// Reuse reader for next region
								let download = download_region(reader, key + 1, this.len);
								let _ = this.content_cache.insert((key + 1, DataRegion::Download(Box::pin(download))));
							}
						},

						std::task::Poll::Pending => return std::task::Poll::Pending,
					},

					DataRegion::Downloaded(content) => break content,
				}
			}
			else {
				let response = this.api.download(&this.release, &this.user_credentials, Some(&format!("bytes={}-", key * REGION_LEN_MAX as u64)));
				let reader =
					futures_util::stream::TryStreamExt::into_async_read(
						Box::pin(
							futures_util::stream::TryStreamExt::map_err(
								response,
								|err| std::io::Error::new(std::io::ErrorKind::Other, err))) as _);
				let download = download_region(reader, key, this.len);
				let _ = this.content_cache.insert((key, DataRegion::Download(Box::pin(download))));
			}
		};

		let read = std::io::Read::read(&mut &content[offset..], buf)?;
		self.pos += read as u64;
		std::task::Poll::Ready(Ok(read))
	}
}

impl futures_util::io::AsyncSeek for WebReader<'_> {
	fn poll_seek(mut self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>, pos: std::io::SeekFrom) -> std::task::Poll<std::io::Result<u64>> {
		let (base, offset) = match pos {
			std::io::SeekFrom::Start(start) => (start, 0),
			std::io::SeekFrom::End(end) => (self.len, end),
			std::io::SeekFrom::Current(current) => (self.pos, current),
		};

		let new_pos =
			if offset >= 0 {
				base.checked_add(offset as u64)
			}
			else {
				base.checked_sub(offset.wrapping_neg() as u64)
			};

		self.pos = new_pos.ok_or_else(|| std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"invalid seek to a negative or overflowing position"))?;

		std::task::Poll::Ready(Ok(self.pos))
	}
}

async fn download_region<R>(mut reader: R, key: u64, file_len: u64) -> std::io::Result<(R, Vec<u8>)> where R: futures_util::io::AsyncRead + Unpin {
	let region_len = std::cmp::min((key + 1).saturating_mul(REGION_LEN_MAX as u64), file_len) - key * REGION_LEN_MAX as u64;
	let mut buf = vec![0_u8; region_len as usize];
	futures_util::io::AsyncReadExt::read_exact(&mut reader, &mut buf).await?;
	Ok((reader, buf))
}
