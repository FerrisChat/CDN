pub use ferrischat_common::types::ErrorJson;

use axum::body::{self, BoxBody};
use axum::extract::multipart::MultipartError as AxumMultipartError;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;

use tokio::task::JoinError;

use serde::{Deserialize, Serialize};
use std::io::Error as IoError;

use simd_json::to_vec;

use reqwest::Error as ReqwestError;

use redis::RedisError;

use deadpool::managed::PoolError;

use http::header::CONTENT_TYPE;
use http::HeaderValue;

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadResponse {
    pub url: String,
}

pub enum CdnError {
    Http(ErrorJson),
    Default,
    FileSizeExceeded,
    NoFileName,
    MultipartError(AxumMultipartError),
    FailedToOpenRedisConnection(PoolError<RedisError>),
    FailedToGetNode,
    NotFound,
    FailedToDeCompress(IoError),
    FailedToOpen(IoError),
    RequestFailed(String, u16),
    ReqwestFailed(ReqwestError),
    NoFileExtension,
    FailedToCompress(IoError),
    FailedToSpawnBlock(JoinError),
    FailedToSave(IoError),
    NoFile,
}

impl From<ErrorJson> for CdnError {
    fn from(err: ErrorJson) -> Self {
        Self::Http(err)
    }
}

impl IntoResponse for CdnError {
    fn into_response(self) -> Response<BoxBody> {
        let (body, status_code) = match self {
            CdnError::Http(err) => (err, StatusCode::BAD_REQUEST),
            CdnError::Default => (
                ErrorJson::new_500(
                    "Something went wrong while trying to save the file.".to_string(),
                    true,
                    None,
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FileSizeExceeded => (
                ErrorJson::new_400(
                    "File size exceeded. Please try again with a smaller file.".to_string(),
                ),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            CdnError::NoFileName => (
                ErrorJson::new_400("No file name provided".to_string()),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::MultipartError(err) => (
                ErrorJson::new_400(format!("Failed to parse multipart: {:?}", err)),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::FailedToOpenRedisConnection(err) => (
                ErrorJson::new_500(
                    format!("Failed to open redis connection: {:?}", err),
                    true,
                    None,
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToGetNode => (
                ErrorJson::new_500("Failed to get node from redis.".to_string(), true, None),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NotFound => (
                ErrorJson::new_404("File not found".to_string()),
                StatusCode::NOT_FOUND,
            ),
            CdnError::FailedToDeCompress(err) => (
                ErrorJson::new_500(format!("Failed to decompress: {:?}", err), true, None),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToOpen(err) => (
                ErrorJson::new_500(format!("Failed to open file: {:?}", err), true, None),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::RequestFailed(err, code) => (
                ErrorJson::new_500(format!("Request failed: {}", err), true, None),
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            ),
            CdnError::ReqwestFailed(err) => (
                ErrorJson::new_500(format!("Reqwest failed: {:?}", err), true, None),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NoFileExtension => (
                ErrorJson::new_400("No file extension provided".to_string()),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::FailedToCompress(err) => (
                ErrorJson::new_500(
                    format!("Failed to compress the file: {:?}", err),
                    true,
                    None,
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToSpawnBlock(err) => (
                ErrorJson::new_500(
                    format!("Task failed to execute to completion: {:?}", err),
                    true,
                    None,
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToSave(err) => (
                ErrorJson::new_500(format!("Failed to save the file: {:?}", err), true, None),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NoFile => (
                ErrorJson::new_400("No file provided".to_string()),
                StatusCode::BAD_REQUEST,
            ),
        };

        let bytes = match to_vec(&body) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
                    .body(body::boxed(body::Full::from(err.to_string())))
                    .expect("failed to convert static data to a valid request");
            }
        };

        Response::builder()
            .status(status_code)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(body::boxed(body::Full::from(bytes)))
            .unwrap_or_else(|e| {
                // this should only be reachable if a invalid HTTP code is passed in
                unreachable!(
                    "got an error while attempting to construct HTTP response for ServerError: {}",
                    e
                )
            })
    }
}
