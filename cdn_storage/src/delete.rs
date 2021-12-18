use cdn_common::CdnError;

use axum::extract::Path;

use http::StatusCode;

use crate::config::STORAGE_PATH;

use tokio::fs;

use std::path::PathBuf;

pub async fn delete(Path(filename): Path<String>) -> Result<StatusCode, CdnError> {
    let path = PathBuf::from(format!("{}/{}", *STORAGE_PATH, filename));

    if !path.exists() {
        debug!("File not found: {:?}", path);
        return Err(CdnError::NotFound);
    }

    fs::remove_file(path)
        .await
        .map_err(|e| CdnError::FailedToOpen(e))?;

    Ok(StatusCode::NO_CONTENT)
}
