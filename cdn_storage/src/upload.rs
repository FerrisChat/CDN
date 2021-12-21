use axum::extract::Multipart;
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
        let mut file_size: u64 = 0;
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(CdnError::MultipartError)?;

            file_size += data.len() as u64;

            let max_content_length: u64 = get_max_content_length().await?;

            if file_size > max_content_length {
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
            .ok_or(CdnError::NoFileName)?
            .split('.')
            .last()
            .ok_or(CdnError::NoFileExtension)?;

        let path = PathBuf::from(format!("{}/{}.{}", *STORAGE_PATH, file_hash, ext));

        let node_id = get_node_id();

        if path.exists() {
            return Ok(Json(UploadResponse {
                url: format!("{}/uploads/{}/{}.{}", *HOST, node_id, file_hash, ext),
            }));
        }

        let mut encoder = ZstdEncoder::with_quality(Vec::new(), Level::Best);
        encoder
            .write_all(&buffer)
            .await
            .map_err(CdnError::FailedToCompress)?;
        encoder
            .shutdown()
            .await
            .map_err(CdnError::FailedToCompress)?;

        fs::write(path, &encoder.into_inner()[..])
            .await
            .map_err(CdnError::FailedToSave)?;

        Ok(Json(UploadResponse {
            url: format!("{}/uploads/{}/{}.{}", *HOST, node_id, file_hash, ext),
        }))
    } else {
        Err(CdnError::NoFile)
    }
}
