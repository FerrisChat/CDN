use cdn_common::CdnError;

use axum::body::{self, BoxBody};
use axum::extract::Path;

use http::header::CONTENT_TYPE;
use http::HeaderValue;
use http::{Response, StatusCode};

use async_compression::tokio::write::ZstdDecoder;
use tokio::io::AsyncWriteExt as _; // for `write_all` and `shutdown`

use crate::http::get_file;
use crate::node::get_node_ip;

pub async fn download(
    Path((node, filename)): Path<(String, String)>,
) -> Result<Response<BoxBody>, CdnError> {
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

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )
        .body(body::boxed(body::Full::from(decoder.into_inner())))
        .unwrap_or_else(|e| {
            // this should only be reachable if a invalid HTTP code is passed in
            unreachable!(
                "got an error while attempting to construct HTTP response for ServerError: {}",
                e
            )
        });

    Ok(resp)
}
