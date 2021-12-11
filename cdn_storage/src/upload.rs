use axum::extract::{ContentLengthLimit, Multipart};
use axum::Json;

use async_compression::{tokio::write::ZstdEncoder, Level};
use cdn_common::{CdnError, UploadResponse};
use futures::stream::StreamExt;
use hmac_sha512::Hash;
use std::path::PathBuf;

use tokio::{fs, io::AsyncWriteExt as _}; // for `write_all` and `shutdown`

const MAX_CONTENT_LENGTH: usize = 1024 * 1024 * 10;

pub async fn upload(
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { MAX_CONTENT_LENGTH as u64 }>,
) -> Result<Json<UploadResponse>, CdnError> {
    if let Ok(Some(mut field)) = multipart.next_field().await {
        let mut file_size: usize = 0;
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| CdnError::MultipartError(e))?;

            file_size += data.len();

            if file_size > MAX_CONTENT_LENGTH {
                return Err(CdnError::FileSizeExceeded);
            }

            buffer.append(&mut data.to_vec());
        }

        let hash = tokio::task::block_in_place(move || Hash::hash(&buffer[..]));

        let file_hash = hash.iter().map(|x| format!("{:02x}", x)).collect::<String>();

        let ext = field
            .file_name()
            .ok_or_else(|| CdnError::NoFileName)?
            .split('.')
            .last()
            .ok_or_else(|| CdnError::NoFileExtension)?;

        let path = PathBuf::from(format!("/etc/ferrischat/CDN/uploads/{}.{}", file_hash, ext));

        if path.exists() {
            return Ok(Json(
                UploadResponse {
                    url: format!(
                        "https://cdn.ferrischat.com/node/uploads/{}.{}",
                        file_hash, ext
                    ),
                }
                .into(),
            ));
        }

        let mut encoder = ZstdEncoder::with_quality(Vec::new(), Level::Best);
        encoder
            .write_all(&buffer)
            .await
            .map_err(|e| CdnError::FailedToCompress(e))?;
        encoder
            .shutdown()
            .await
            .map_err(|e| CdnError::FailedToCompress(e))?;

        fs::write(path, &encoder.into_inner()[..])
            .await
            .map_err(|e| CdnError::FailedToSave(e))?;

        Ok(Json(
            UploadResponse {
                url: format!(
                    "https://cdn.ferrischat.com/node/uploads/{}.{}",
                    file_hash, ext
                ),
            }
            .into(),
        ))
    } else {
        Err(CdnError::NoFile)
    }
}
