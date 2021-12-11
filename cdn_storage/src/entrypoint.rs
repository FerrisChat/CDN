use crate::{download, upload};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;

use std::io::ErrorKind::AlreadyExists;

use crate::config::STORAGE_PATH;

use std::fs;

pub async fn entrypoint() {
    fs::create_dir_all(STORAGE_PATH.to_string())
        .map_err(|e| match e {
            AlreadyExists => (),
            _ => panic!("Failed to create uploads directory: {:?}", e)
        });

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
