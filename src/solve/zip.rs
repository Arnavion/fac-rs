#![allow(
	clippy::cast_possible_truncation,
	clippy::naive_bytecount,
)]

use futures_util::io::{ AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt };

#[derive(Debug, failure::Fail)]
pub(super) enum Error {
	#[fail(display = "central-directory record #{} is corrupt", _0)]
	CentralDirectoryEntryCorrupt(usize, #[cause] FileMetaCorruptReason),

	#[fail(display = "end-of-central-directory record is corrupt")]
	EndOfCentralDirectorRecordCorrupt,

	#[fail(display = "could not find end-of-central-directory record")]
	EndOfCentralDirectorRecordNotFound,

	#[fail(display = "info.json is corrupt")]
	FileCorrupt,

	#[fail(display = "info.json could not be parsed")]
	FileInvalidJson(#[cause] serde_json::Error),

	#[fail(display = "info.json local-header record is corrupt")]
	FileLocalHeaderCorrupt(#[cause] FileMetaCorruptReason),

	#[fail(display = "info.json file-local-header record has different metadata than its central-directory-entry record")]
	FileMetadataCorrupt,

	#[fail(display = "info.json not found")]
	FileNotFound,

	#[fail(display = "i/o error")]
	Io(#[cause] std::io::Error),

	#[fail(display = "info.json is compressed with method {} but only Deflated and Stored are supported", _0)]
	UnsupportedCompressionMethod(zip::CompressionMethod),
}

#[derive(Debug, failure::Fail)]
pub(super) enum FileMetaCorruptReason {
	#[fail(display = "missing magic")]
	MissingMagic,
}

pub(super) async fn find_info_json(
	reader: &mut (impl AsyncRead + AsyncSeek + Unpin),
) -> Result<factorio_mods_local::ModInfo, Error> {
	// PKZIP spec: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT

	// Find the end-of-central-directory record
	//
	// This is at the end of the file, and starts with a magic value. It is at least 22 bytes long, and is followed by a variable-length comment field
	// that can be between 0 and `u16::max_value()` bytes long (inclusive).
	//
	// So start at EOF - 22 and work backwards to find it.

	const EOCD_MIN_LEN: u64 = 22;

	let file_len = reader.seek(std::io::SeekFrom::End(0)).await.map_err(Error::Io)?;
	let eocd_start_pos_min = file_len.saturating_sub(EOCD_MIN_LEN + u64::from(u16::max_value()));

	let mut eocd_start_pos = file_len.checked_sub(EOCD_MIN_LEN).ok_or_else(|| Error::EndOfCentralDirectorRecordNotFound)?;

	let (central_directory_pos, num_central_directory_entries) = loop {
		reader.seek(std::io::SeekFrom::Start(eocd_start_pos)).await.map_err(Error::Io)?;
		if read_u32_le(reader).await? == 0x0605_4b50 {
			// Seek to comment length and parse it
			reader.seek(std::io::SeekFrom::Current(16)).await.map_err(Error::Io)?;
			let comment_len = u64::from(read_u16_le(reader).await?);

			// Ensure that the comment corresponding to this length would extend to the end of the file
			if eocd_start_pos + EOCD_MIN_LEN + comment_len != file_len {
				continue;
			}

			// This looks valid
			reader.seek(std::io::SeekFrom::Start(eocd_start_pos + 8)).await.map_err(Error::Io)?;
			let num_central_directory_entries = usize::from(read_u16_le(reader).await?);
			reader.seek(std::io::SeekFrom::Current(6)).await.map_err(Error::Io)?;
			let central_directory_pos = u64::from(read_u32_le(reader).await?);
			break (central_directory_pos, num_central_directory_entries);
		}

		if eocd_start_pos == eocd_start_pos_min {
			return Err(Error::EndOfCentralDirectorRecordNotFound);
		}

		eocd_start_pos = eocd_start_pos.saturating_sub(1);
	};

	if central_directory_pos >= eocd_start_pos {
		return Err(Error::EndOfCentralDirectorRecordCorrupt);
	}

	reader.seek(std::io::SeekFrom::Start(central_directory_pos)).await.map_err(Error::Io)?;

	if num_central_directory_entries == 0 {
		return Err(Error::FileNotFound);
	}

	let mut info_json_entry = None;
	for i in 0..num_central_directory_entries {
		let entry = CentralDirectoryEntry::parse(reader, i).await?;
		if entry.file_meta.filename.ends_with(b"/info.json") && entry.file_meta.filename.iter().filter(|&&b| b == b'/').count() == 1 {
			info_json_entry = Some(entry);
			break;
		}
	}
	let info_json_entry = info_json_entry.ok_or_else(|| Error::FileNotFound)?;

	reader.seek(std::io::SeekFrom::Start(info_json_entry.local_header_pos)).await.map_err(Error::Io)?;

	let info_json_file_local_header = FileLocalHeader::parse(reader).await?;

	if info_json_file_local_header.0 != info_json_entry.file_meta {
		return Err(Error::FileMetadataCorrupt);
	}

	let mut buf = vec![0_u8; info_json_file_local_header.0.compressed_size as usize];
	reader.read_exact(&mut buf).await.map_err(Error::Io)?;

	let reader = Reader::new(info_json_entry.file_meta.compression_method, buf, info_json_file_local_header.0.crc32)?;
	let info_json = serde_json::from_reader(reader).map_err(Error::FileInvalidJson)?;
	Ok(info_json)
}

#[derive(Debug, PartialEq)]
struct FileMeta {
	filename: Vec<u8>,
	compression_method: zip::CompressionMethod,
	crc32: u32,
	compressed_size: u64,
	uncompressed_size: u64,
}

#[derive(Debug)]
struct CentralDirectoryEntry {
	file_meta: FileMeta,
	local_header_pos: u64,
}

impl CentralDirectoryEntry {
	async fn parse(reader: &mut (impl AsyncRead + AsyncSeek + Unpin), i: usize) -> Result<Self, Error> {
		if read_u32_le(reader).await? != 0x0201_4b50 {
			return Err(Error::CentralDirectoryEntryCorrupt(i + 1, FileMetaCorruptReason::MissingMagic));
		}

		reader.seek(std::io::SeekFrom::Current(6)).await.map_err(Error::Io)?;

		let compression_method = zip::CompressionMethod::from_u16(read_u16_le(reader).await?);

		reader.seek(std::io::SeekFrom::Current(4)).await.map_err(Error::Io)?;

		let crc32 = read_u32_le(reader).await?;
		let compressed_size = u64::from(read_u32_le(reader).await?);
		let uncompressed_size = u64::from(read_u32_le(reader).await?);
		let filename_len = usize::from(read_u16_le(reader).await?);
		let extra_field_len = i64::from(read_u16_le(reader).await?);
		let file_comment_len = i64::from(read_u16_le(reader).await?);

		reader.seek(std::io::SeekFrom::Current(8)).await.map_err(Error::Io)?;

		let local_header_pos = u64::from(read_u32_le(reader).await?);

		let mut filename = vec![0_u8; filename_len];
		reader.read_exact(&mut filename).await.map_err(Error::Io)?;

		let result = CentralDirectoryEntry {
			file_meta: FileMeta {
				filename,
				compression_method,
				crc32,
				compressed_size,
				uncompressed_size,
			},
			local_header_pos,
		};

		reader.seek(std::io::SeekFrom::Current(extra_field_len + file_comment_len)).await.map_err(Error::Io)?;

		Ok(result)
	}
}

#[derive(Debug)]
struct FileLocalHeader(FileMeta);

impl FileLocalHeader {
	async fn parse(reader: &mut (impl AsyncRead + AsyncSeek + Unpin)) -> Result<Self, Error> {
		if read_u32_le(reader).await? != 0x0403_4b50 {
			return Err(Error::FileLocalHeaderCorrupt(FileMetaCorruptReason::MissingMagic));
		}

		reader.seek(std::io::SeekFrom::Current(4)).await.map_err(Error::Io)?;

		let compression_method = zip::CompressionMethod::from_u16(read_u16_le(reader).await?);

		reader.seek(std::io::SeekFrom::Current(4)).await.map_err(Error::Io)?;

		let crc32 = read_u32_le(reader).await?;
		let compressed_size = u64::from(read_u32_le(reader).await?);
		let uncompressed_size = u64::from(read_u32_le(reader).await?);
		let filename_len = usize::from(read_u16_le(reader).await?);
		let extra_field_len = i64::from(read_u16_le(reader).await?);

		let mut filename = vec![0_u8; filename_len];
		reader.read_exact(&mut filename).await.map_err(Error::Io)?;

		let result = FileLocalHeader(FileMeta {
			filename,
			compression_method,
			compressed_size,
			uncompressed_size,
			crc32,
		});

		reader.seek(std::io::SeekFrom::Current(extra_field_len)).await.map_err(Error::Io)?;

		Ok(result)
	}
}

async fn read_u16_le(reader: &mut (impl AsyncRead + Unpin)) -> Result<u16, Error> {
	let mut buf = [0_u8; 2];
	reader.read_exact(&mut buf).await.map_err(Error::Io)?;
	Ok(u16::from_le_bytes(buf))
}

async fn read_u32_le(reader: &mut (impl AsyncRead + Unpin)) -> Result<u32, Error> {
	let mut buf = [0_u8; 4];
	reader.read_exact(&mut buf).await.map_err(Error::Io)?;
	Ok(u32::from_le_bytes(buf))
}

#[derive(Debug)]
struct Reader {
	inner: ReaderInner,
	hasher: crc32fast::Hasher,
	expected_crc32: u32,
}

#[derive(Debug)]
enum ReaderInner {
	Deflated(libflate::deflate::Decoder<std::io::Cursor<Vec<u8>>>),
	Stored(std::io::Cursor<Vec<u8>>),
}

impl Reader {
	fn new(compression_method: zip::CompressionMethod, data: Vec<u8>, expected_crc32: u32) -> Result<Self, Error> {
		let inner = match compression_method {
			zip::CompressionMethod::Deflated =>
				ReaderInner::Deflated(libflate::deflate::Decoder::new(std::io::Cursor::new(data))),

			zip::CompressionMethod::Stored =>
				ReaderInner::Stored(std::io::Cursor::new(data)),

			compression_method => return Err(Error::UnsupportedCompressionMethod(compression_method)),
		};

		Ok(Reader {
			inner,
			hasher: Default::default(),
			expected_crc32,
		})
	}
}

impl std::io::Read for Reader {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let result = match &mut self.inner {
			ReaderInner::Deflated(reader) => std::io::Read::read(reader, buf)?,
			ReaderInner::Stored(reader) => std::io::Read::read(reader, buf)?,
		};

		self.hasher.update(&buf[..result]);

		if result == 0 {
			let crc32 = self.hasher.clone().finalize();
			if crc32 != self.expected_crc32 {
				return Err(super::io_error_from_fail(&Error::FileCorrupt));
			}
		}

		Ok(result)
	}
}
