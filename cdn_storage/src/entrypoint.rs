use crate::{download, upload};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;

use std::io::ErrorKind::AlreadyExists;

use crate::config::STORAGE_PATH;
use crate::node::load_redis;

use std::fs;

pub async fn entrypoint(node_id: u64) {
    let _ = fs::create_dir_all(STORAGE_PATH.to_string()).map_err(|e| match e.kind() {
        AlreadyExists => (),
        _ => panic!("Failed to create uploads directory: {:?}", e),
    });

    load_redis(node_id).await;

    let router = Router::new()
        .route("/ping", get(async || (StatusCode::OK, "")))
        .route("/upload", post(upload))
        .route("/download/:filename", get(download));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));

    let server = axum::Server::bind(&addr).serve(router.into_make_service()).with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to wait for ctrl+c: you will need to SIGTERM the server if you want it to shut down");
    });

    server.await.expect("failed to start HTTP server");
}
