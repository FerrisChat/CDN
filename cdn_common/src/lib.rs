pub use ferrischat_common::types::ErrorJson;

use axum::body::{self, BoxBody};
use axum::http::Response;
use axum::response::IntoResponse;

use http::header::CONTENT_TYPE;
use http::HeaderValue;

use simd_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadResponse {
    pub url: String,
}

pub enum CdnError {
    http(ErrorJson),
    default,
}

impl From<ErrorJson> for CdnError {
    fn from(err: ErrorJson) -> Self {
        Self::http(err)
    }
}

impl IntoResponse for CdnError {
    fn into_response(self) -> Response<BoxBody> {
        let body = match self {
            CdnError::http(err) => err,
            CdnError::default => ErrorJson::new_500(
                "Something went wrong while trying to save the file.".to_string(),
                true,
                None,
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

        axum::http::Response::builder()
            .status(body.get_code())
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
