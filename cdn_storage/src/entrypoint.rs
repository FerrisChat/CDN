use crate::{download, upload};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::config::STORAGE_PATH;

use tokio::fs;

#[allow(clippy::expect_used)]
pub async fn entrypoint() {
    let path = Path::from(*STORAGE_PATH.clone());

    if !path.exists() {
        fs::create_dir(path)
            .await
            .expect("Failed to create uploads directory");
    }

    let router = Router::new()
        .route("/ping", get(async || (StatusCode::OK, "")))
        .route("/upload", post(upload))
        .route("/download/:filename", get(download));

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));

    let server = axum::Server::bind(&addr).serve(router.into_make_service()).with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to wait for ctrl+c: you will need to SIGTERM the server if you want it to shut down");
    });

    server.await.expect("failed to start HTTP server");
}
