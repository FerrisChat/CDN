use cdn_common::CdnError;

use axum::body::{self, BoxBody};
use axum::extract::Path;

use crate::config::CACHE;

use http::header::CONTENT_TYPE;
use http::HeaderValue;
use http::{Response, StatusCode};

use tree_magic;

use async_compression::tokio::write::ZstdDecoder;
use tokio::io::AsyncWriteExt as _; // for `write_all` and `shutdown`

use crate::cache::{get_from_cache, insert_into_cache};
use crate::http::get_file;
use crate::node::get_node_ip;

pub async fn download(
    Path((node, filename)): Path<(String, String)>,
) -> Result<Response<BoxBody>, CdnError> {
    if *CACHE {
        if let Some(file_) = get_from_cache(&filename) {
            let content_type = tree_magic::from_u8(&file_);
            let decoded_file = file_;

            let resp = Response::builder()
                .status(StatusCode::FOUND)
                .header(
                    CONTENT_TYPE,
                    HeaderValue::from_str(content_type.as_str())
                        .unwrap_or(HeaderValue::from_static("application/octet-stream")),
                )
                .body(body::boxed(body::Full::from(decoded_file)))
                .unwrap_or_else(|e| {
                    // this should only be reachable if a invalid HTTP code is passed in
                    unreachable!(
                    "got an error while attempting to construct HTTP response for ServerError: {}",
                    e
                )
                });

            return Ok(resp);
        }
    }

    let node_ip = get_node_ip(node).await?;

    let file = get_file(node_ip, filename.clone()).await?;

    let mut decoder = ZstdDecoder::new(Vec::new());
    decoder
        .write_all(&file)
        .await
        .map_err(CdnError::FailedToDeCompress)?;
    decoder
        .shutdown()
        .await
        .map_err(CdnError::FailedToDeCompress)?;

    let decoded_file = decoder.into_inner();
    let content_type = tree_magic::from_u8(&decoded_file);

    if *CACHE {
        insert_into_cache(filename, decoded_file.clone(), decoded_file.len() as i64).await;
    }

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type.as_str())
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        )
        .body(body::boxed(body::Full::from(decoded_file)))
        .unwrap_or_else(|e| {
            // this should only be reachable if a invalid HTTP code is passed in
            unreachable!(
                "got an error while attempting to construct HTTP response for ServerError: {}",
                e
            )
        });

    Ok(resp)
}
