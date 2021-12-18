use crate::{delete as delete_route, download as download_route, upload as upload_route};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::Router;
use std::net::SocketAddr;

use tower_http::trace::TraceLayer;

use crate::http::load_http;
use crate::node::load_redis;

pub async fn entrypoint() {
    load_redis().await;
    load_http();

    let router = Router::new()
        .route("/ping", get(async || (StatusCode::OK, "")))
        .route("/upload", post(upload_route))
        .route("/uploads/:node/*filename", get(download_route))
        .route("/uploads/:node/*filename", delete(delete_route))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));

    let server = axum::Server::bind(&addr).serve(router.into_make_service()).with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to wait for ctrl+c: you will need to SIGTERM the server if you want it to shut down");
    });

    server.await.expect("failed to start HTTP server");
}
