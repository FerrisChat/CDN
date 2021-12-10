use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post};
use crate::upload;


#[allow(clippy::expect_used)]
pub async fn entrypoint() {
    let router = Router::new()
        .route("/ping", get(async || (StatusCode::OK, "")))
        .route("/upload", get(upload))
        .route("/download/:filename", post(download));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await?;
    let acceptor = hyper::server::accept::from_stream(stream);

    let server = axum::Server::builder(acceptor).serve(router.into_make_service()).with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to wait for ctrl+c: you will need to SIGTERM the server if you want it to shut down");
    });

    server.await.expect("failed to start HTTP server");
}
