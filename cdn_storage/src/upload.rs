use axum::extract::{ContentLengthLimit, Multipart};
use axum::Json;

use async_compression::{tokio::write::ZstdEncoder, Level};
use cdn_common::{CdnError, UploadResponse};
use futures::stream::StreamExt;
use hmac_sha512::Hash;
use std::path::PathBuf;

use crate::node::{get_max_content_length, get_node_id};

use crate::config::{HOST, STORAGE_PATH};

use tokio::{fs, io::AsyncWriteExt as _}; // for `write_all` and `shutdown`

pub async fn upload(mut multipart: Multipart) -> Result<Json<UploadResponse>, CdnError> {
    if let Ok(Some(mut field)) = multipart.next_field().await {
        let mut file_size: usize = 0;
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| CdnError::MultipartError(e))?;

            file_size += data.len();

            if file_size > get_max_content_length().await {
                return Err(CdnError::FileSizeExceeded);
            }

            buffer.append(&mut data.to_vec());
        }

        let hash = tokio::task::block_in_place(|| Hash::hash(&buffer[..]));

        let file_hash = hash
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>();

        let ext = field
            .file_name()
            .ok_or_else(|| CdnError::NoFileName)?
            .split('.')
            .last()
            .ok_or_else(|| CdnError::NoFileExtension)?;

        let path = PathBuf::from(format!("{}/{}.{}", *STORAGE_PATH, file_hash, ext));

        let node_id = get_node_id();

        if path.exists() {
            return Ok(Json(
                UploadResponse {
                    url: format!("{}/{}/uploads/{}.{}", *HOST, node_id, file_hash, ext),
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
                url: format!("{}/{}/uploads/{}.{}", *HOST, node_id, file_hash, ext),
            }
            .into(),
        ))
    } else {
        Err(CdnError::NoFile)
    }
}
