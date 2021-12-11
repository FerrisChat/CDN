use axum::extract::{ContentLengthLimit, Multipart};
use axum::Json;

use async_compression::{tokio::write::ZstdEncoder, Level};
use cdn_common::{CdnError, UploadResponse};
use futures::stream::StreamExt;
use hmac_sha512::Hash;
use std::path::Path;

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

        let hash = tokio::task::spawn_blocking(move || Hash::hash(&buffer[..])).await;

        let file_hash = hash.iter().map(|x| format!("{:02x}", x)).collect::<String>();

        let ext = field
            .file_name()
            .ok_or_else(|| CdnError::NoFileName)?
            .split('.')
            .last()
            .ok_or_else(|| CdnError::NoFileExtension)?;

        let path = Path::new(format!("/etc/ferrischat/CDN/uploads/{}.{}", file_hash, ext).as_ref());

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

        let compressed: Vec<u8> = Vec::new();

        let mut encoder = ZstdEncoder::with_quality(compressed, Level::Best);
        encoder
            .write_all(&buffer)
            .await
            .map_err(|e| CdnError::FailedToCompress(e))?;
        encoder
            .shutdown()
            .await
            .map_err(|e| CdnError::FailedToCompress(e))?;

        fs::write(path, &compressed[..])
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
