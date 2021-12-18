use cdn_auth::Authorization;
use cdn_common::CdnError;

use axum::extract::Path;

use http::StatusCode;

use crate::http::delete_file;
use crate::node::get_node_ip;

pub async fn delete(
    _: Authorization,
    Path((node, filename)): Path<(String, String)>,
) -> Result<StatusCode, CdnError> {
    let node_ip = get_node_ip(node).await?;

    delete_file(node_ip, filename).await?;

    Ok(StatusCode::NO_CONTENT)
}
