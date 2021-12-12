pub use ferrischat_common::types::ErrorJson;

use axum::body::{self, BoxBody};
use axum::extract::multipart::MultipartError as AxumMultipartError;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;

use tokio::task::JoinError;

use serde::{Deserialize, Serialize};
use std::io::Error as IoError;

use reqwest::Error as ReqwestError;

use redis::RedisError;

use deadpool::managed::PoolError;

use http::header::CONTENT_TYPE;
use http::HeaderValue;

use simd_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadResponse {
    pub url: String,
}

use argon2_async::Error as Argon2AsyncError;
use sqlx::Error as SqlxError;

pub enum VerifyTokenFailure {
    MissingDatabase,
    InvalidToken,
    DbError(SqlxError),
    VerifierError(Argon2AsyncError),
}

impl From<SqlxError> for VerifyTokenFailure {
    #[inline]
    fn from(e: sqlx::Error) -> Self {
        Self::DbError(e)
    }
}

impl From<Argon2AsyncError> for VerifyTokenFailure {
    fn from(e: argon2_async::Error) -> Self {
        Self::VerifierError(e)
    }
}

use base64::DecodeError;
use std::string::FromUtf8Error;

/// Errors returned when splitting a token into its constituent parts.
pub enum SplitTokenError {
    /// Invalid UTF-8 detected
    InvalidUtf8(FromUtf8Error),
    /// Invalid base64 encoded data detected
    Base64DecodeError(DecodeError),
    /// Invalid integer found in the base64 encoded data.
    InvalidInteger(std::num::ParseIntError),
    /// Parts of the token are missing.
    ///
    /// The attached integer shows what part is missing. Zero-indexed.
    MissingParts(u8),
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
        Self::Http(err.into())
    }
}

impl From<VerifyTokenFailure> for CdnError {
    fn from(e: VerifyTokenFailure) -> Self {
        let reason = match e {
            VerifyTokenFailure::MissingDatabase => "database pool not found".to_string(),
            VerifyTokenFailure::DbError(e) => return Self::from(e),
            VerifyTokenFailure::VerifierError(e) => {
                format!("argon2 verifier returned an error: {}", e)
            }
            VerifyTokenFailure::InvalidToken => {
                unreachable!("a invalid token error should be handled earlier")
            }
        };
        Self::Http(ErrorJson::new_500(reason, false, None))
    }
}

impl From<Argon2Error> for CdnError {
    fn from(e: Argon2Error) -> Self {
        let reason = format!(
            "hashing error: {}",
            match e {
                Argon2AsyncError::Communication => {
                    "an error was encountered while waiting for a background thread to complete."
                        .to_string()
                }
                Argon2AsyncError::Argon(e) =>
                    format!("underlying argon2 algorithm threw an error: {}", e),
                Argon2AsyncError::PasswordHash(e) => {
                    format!("password string handling library threw an error: {}", e)
                }
                Argon2AsyncError::MissingConfig => "global configuration unset".to_string(),
                _ => "unknown error".to_string(),
            }
        );
        Self::Http(ErrorJson::new_500(reason, false, None))
    }
}

impl From<SplitTokenError> for CdnError {
    fn from(e: SplitTokenError) -> Self {
        let message = match e {
            SplitTokenError::InvalidUtf8(e) => format!("invalid utf8 found in token: {}", e),
            SplitTokenError::Base64DecodeError(e) => {
                format!("invalid base64 data found in token: {}", e)
            }
            SplitTokenError::InvalidInteger(e) => format!("invalid integer found in token: {}", e),
            SplitTokenError::MissingParts(idx) => format!("part {} of token missing", idx),
        };
        Self::Http(ErrorJson::new_400(message))
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
                )
                .into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FileSizeExceeded => (
                ErrorJson::new_400(
                    "File size exceeded. Please try again with a smaller file.".to_string(),
                )
                .into(),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            CdnError::NoFileName => (
                ErrorJson::new_400("No file name provided".to_string()).into(),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::MultipartError(err) => (
                ErrorJson::new_400(format!("Failed to parse multipart: {:?}", err)).into(),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::FailedToOpenRedisConnection(err) => (
                ErrorJson::new_500(
                    format!("Failed to open redis connection: {:?}", err),
                    true,
                    None,
                )
                .into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToGetNode => (
                ErrorJson::new_500("Failed to get node from redis.".to_string(), true, None).into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NotFound => (
                ErrorJson::new_404("File not found".to_string()).into(),
                StatusCode::NOT_FOUND,
            ),
            CdnError::FailedToDeCompress(err) => (
                ErrorJson::new_500(format!("Failed to decompress: {:?}", err), true, None).into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToOpen(err) => (
                ErrorJson::new_500(format!("Failed to open file: {:?}", err), true, None).into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::RequestFailed(err, code) => (
                ErrorJson::new_500(format!("Request failed: {}", err), true, None).into(),
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            ),
            CdnError::ReqwestFailed(err) => (
                ErrorJson::new_500(format!("Reqwest failed: {:?}", err), true, None).into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NoFileExtension => (
                ErrorJson::new_400("No file extension provided".to_string()).into(),
                StatusCode::BAD_REQUEST,
            ),
            CdnError::FailedToCompress(err) => (
                ErrorJson::new_500(
                    format!("Failed to compress the file: {:?}", err),
                    true,
                    None,
                )
                .into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToSpawnBlock(err) => (
                ErrorJson::new_500(
                    format!("Task failed to execute to completion: {:?}", err),
                    true,
                    None,
                )
                .into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::FailedToSave(err) => (
                ErrorJson::new_500(format!("Failed to save the file: {:?}", err), true, None)
                    .into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            CdnError::NoFile => (
                ErrorJson::new_400("No file provided".to_string()).into(),
                StatusCode::BAD_REQUEST,
            ),
        };

        let bytes = match simd_json::to_vec(&body) {
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
