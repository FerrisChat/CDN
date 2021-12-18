use crate::{delete as delete_route, download as download_route, upload as upload_route};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::Router;
use std::net::SocketAddr;

use tower_http::trace::TraceLayer;

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
        .route("/upload", post(upload_route))
        .route("/uploads/*filename", get(download_route))
        .route("/uploads/*filename", delete(delete_route))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));

    let server = axum::Server::bind(&addr).serve(router.into_make_service()).with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to wait for ctrl+c: you will need to SIGTERM the server if you want it to shut down");
    });

    server.await.expect("failed to start HTTP server");
}
