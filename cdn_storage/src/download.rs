use cdn_common::CdnError;

use axum::body::{self, BoxBody};
use axum::extract::Path;

use http::header::CONTENT_TYPE;
use http::HeaderValue;
use http::{Response, StatusCode};

use crate::config::STORAGE_PATH;

use tokio::fs;

use std::path::PathBuf;

pub async fn download(Path(filename): Path<String>) -> Result<Response<BoxBody>, CdnError> {
    let path = PathBuf::from(format!("{}/{}", *STORAGE_PATH, filename));

    if !path.exists() {
        debug!("File not found: {:?}", path);
        return Err(CdnError::NotFound);
    }

    let file = fs::read(path).await.map_err(CdnError::FailedToOpen)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )
        .body(body::boxed(body::Full::from(file))) // User facing server will handle decompression
        .unwrap_or_else(|e| {
            // this should only be reachable if a invalid HTTP code is passed in
            unreachable!(
                "got an error while attempting to construct HTTP response for ServerError: {}",
                e
            )
        });

    Ok(resp)
}
