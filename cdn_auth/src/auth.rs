use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequest, RequestParts};
use cdn_common::{CdnError, ErrorJson};
use http::header::AUTHORIZATION;

use crate::config::AUTH_TOKEN;

pub struct Authorization;

#[async_trait]
impl FromRequest<Body> for Authorization {
    type Rejection = CdnError;

    async fn from_request(req: &mut RequestParts<Body>) -> Result<Self, Self::Rejection> {
        let headers = req.headers().ok_or_else(|| ErrorJson::new_500(
            "another extractor took headers".to_string(),
            true,
            Some(
                "https://github.com/FerrisChat/Server/issues/new?assignees=tazz4843&\
                labels=bug&template=api_bug_report.yml&title=%5B500%5D%3A+another+extractor+took+headers"
                    .to_string(),
            ),
        ))?;

        let auth_header = headers
            .get(AUTHORIZATION)
            .ok_or_else(|| ErrorJson::new_400("Authorization header not found".to_string()))?;

        let token = String::from_utf8(auth_header.as_bytes().into()).map_err(|e| {
            ErrorJson::new_400(format!(
                "Authorization header contained invalid UTF-8: {}",
                e
            ))
        })?;

        if token == *AUTH_TOKEN {
            Ok(Self)
        } else {
            Err(ErrorJson::new_401("Authorization header passed was invalid".to_string()).into())
        }
    }
}
