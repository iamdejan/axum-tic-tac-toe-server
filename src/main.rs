use std::{net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    extract::{
        WebSocketUpgrade,
        ws::{WebSocket},
    },
    http::{StatusCode, Version},
    response::IntoResponse,
    routing::{any, get},
};
use axum_server::tls_rustls::RustlsConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let certs_folder_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("self_signed_certs");
    let cert_file_path = certs_folder_path.join("fullchain.crt");
    let key_file_path = certs_folder_path.join("key.pem");
    let config = RustlsConfig::from_pem_file(cert_file_path, key_file_path)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", any(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    let mut server = axum_server::bind_rustls(addr, config);

    server.http_builder().http2().enable_connect_protocol();

    server.serve(app.into_make_service()).await.unwrap();
}

async fn index() -> axum::response::Response {
    return (StatusCode::OK, "Hello world").into_response();
}

async fn ws_handler(ws: WebSocketUpgrade, version: Version) -> axum::response::Response {
    tracing::debug!("Accepted a WebSocket using {version:?}");
    return ws.on_upgrade(handle_socket);
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let text = match msg {
            Ok(text) => text,
            Err(e) => {
                tracing::debug!("Client abruptly disconnected: {e}");
                return;
            }
        };

        let send_result = socket.send(text).await;
        if let Err(e) = send_result {
            tracing::debug!("Client abruptly disconnected: {e}")
        }
    }
}
