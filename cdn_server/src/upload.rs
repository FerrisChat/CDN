use rand::seq::SliceRandom;
use rand::thread_rng;

use axum::extract::Multipart;
use axum::Json;
use futures::stream::StreamExt;

use cdn_auth::Authorization;
use cdn_common::{CdnError, UploadResponse};

use crate::http::upload_file;
use crate::node::{get_all_nodes, get_node_ip};

pub async fn upload(
    _: Authorization,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, CdnError> {
    if let Ok(Some(mut field)) = multipart.next_field().await {
        let mut buffer: Vec<u8> = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(CdnError::MultipartError)?;
            buffer.append(&mut data.to_vec());
        }

        let file_name = field.file_name().ok_or(CdnError::NoFileName)?;

        let nodes = get_all_nodes().await?;
        let node = nodes
            .choose(&mut thread_rng())
            .ok_or(CdnError::FailedToGetNode)?;
        let ip = get_node_ip(node.to_string()).await?;

        Ok(Json(upload_file(ip, file_name.to_string(), buffer).await?))
    } else {
        Err(CdnError::NoFile)
    }
}
