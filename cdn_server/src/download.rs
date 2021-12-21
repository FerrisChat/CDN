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
    let mut content_type: String;

    let mut decoded: Vec<u8>;

    let mut status: StatusCode;

    if *CACHE && let Some(file_) = get_from_cache(&filename) {
        content_type = tree_magic::from_u8(&file_);
        decoded = file_;
        status = StatusCode::FOUND;
    } else {
        let node_ip = get_node_ip(node).await?;

        let file = get_file(node_ip, filename).await?;

        let mut decoder = ZstdDecoder::new(Vec::new());
        decoder
            .write_all(&file)
            .await
            .map_err(|e| CdnError::FailedToDeCompress(e))?;
        decoder
            .shutdown()
            .await
            .map_err(|e| CdnError::FailedToDeCompress(e))?;

        decoded = decoder.into_inner();
        content_type = tree_magic::from_u8(&decoded);
        status = StatusCode::OK;

        if *CACHE {
            insert_into_cache(filename, decoded.clone(), decoded.len() as i64).await;
        }
    }

    let resp = Response::builder()
        .status(status)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type.as_str())
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        )
        .body(body::boxed(body::Full::from(decoded)))
        .unwrap_or_else(|e| {
            // this should only be reachable if a invalid HTTP code is passed in
            unreachable!(
                "got an error while attempting to construct HTTP response for ServerError: {}",
                e
            )
        });

    Ok(resp)
}
