use axum::extract::{ContentLengthLimit, Multipart};
use axum::Json;

use cdn_common::{CdnError, ErrorJson, UploadResponse};
use std::io::Write;
use async_compression::{tokio::write::ZstdEncoder, Level};
use hmac_sha512::Hash;
use std::path::Path;

use tokio::{io::AsyncWriteExt as _, fs}; // for `write_all` and `shutdown`

const MAX_CONTENT_LENGTH: usize = 1024 * 1024 * 10;

pub async fn upload(ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { MAX_CONTENT_LENGTH as u64 }>) -> Result<Json<UploadResponse>, CdnError> {
    if let Ok(Some(mut field)) = multipart.next_field().await {
        let mut file_size: usize = 0;
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            file_size += chunk.len();

            if file_size > MAX_CONTENT_LENGTH {
                return Err(ErrorJson::new_400("File size limit exceeded".to_string()).into());
            }

            buffer.append(&mut chunk.to_vec());
        }

        let hash = tokio::task::spawn_blocking(move || {
            Hash::hash(&buffer[..])
        }).await.unwrap_or_else(|| Err(CdnError::new_500("Failed to hash file".to_string(), true, None).into()));

        let file_hash = String::from(hash);

        let ext = multipart.filename.unwrap_or_else(|| Err(ErrorJson::new_400("No file name provided".to_string()).into()).split('.').last().unwrap_or_else(|| ErrorJson::new_400("No file extension found".to_string()).into()));
        let path = Path::new(format!("/etc/ferrischat/CDN/uploads/{}.{}", file_hash, ext));

        if path.exists() {
            return Ok(Json(UploadResponse { url: format!("https://cdn.ferrischat.com/node/uploads/{}.{}", file_hash, ext) }.into(), 302));
        }

        let compressed: Vec<u8> = Vec::new();

        let mut encoder = ZstdEncoder::with_quality(compressed, Level::Best);
        encoder.write_all(&buffer).await.map_err(|e| Err(ErrorJson::new_500(format!("Something went wrong during compression: {:?}", e), true, None).into()));
        encoder.shutdown.await.map_err(|e| ErrorJson::new_500(format!("Something went wrong during compression: {:?}", e), true, None).into());

        fs::write(path, &compressed[..]).await.map_err(|e| Err(ErrorJson::new_500(format!("Something went wrong while saving file: {:?}", e), true, None).into()));

        Ok(Json(UploadResponse { url: format!("https://cdn.ferrischat.com/node/uploads/{}.{}", file_hash, ext) }.into(), 200))
    }
}
