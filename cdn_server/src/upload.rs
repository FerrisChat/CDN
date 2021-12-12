use rand::thread_rng;
use rand::seq::SliceRandom;

use axum::extract::Multipart;
use axum::Json;

use cdn_common::{CdnError, UploadResponse};

use crate::node::{get_all_nodes, get_node_ip};
use crate::http::upload_file;

pub async fn upload(mut multipart: Multipart) -> Result<UploadResponse, CdnError> {
    if let Ok(Some(mut field)) = multipart.next_field().await {
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| CdnError::MultipartError(e))?;
            buffer.append(&mut data.to_vec());
        }

        let file_name = field.file_name().ok_or_else(|| CdnError::NoFileName)?;

        let nodes = get_all_nodes().await?;
        let node = nodes.choose(&mut thread_rng()).ok_or_else(|| CdnError::FailedToGetNode)?;
        let ip = get_node_ip(node).await?;

        Ok(Json(upload_file(ip, file_name, buffer).await?))
    }
    else {
        Err(CdnError::NoFile)
    }
}
