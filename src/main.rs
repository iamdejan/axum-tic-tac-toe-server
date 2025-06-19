use std::{net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{StatusCode, Version},
    response::IntoResponse,
    routing::{any, get},
};
use axum_server::tls_rustls::RustlsConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod game;
use crate::game::{AppState, Command, CommandType, Room};

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
    let cert_file_path = certs_folder_path.join("cert.pem");
    let key_file_path = certs_folder_path.join("key.pem");
    let config = RustlsConfig::from_pem_file(cert_file_path, key_file_path)
        .await
        .unwrap();

    let app_state = AppState::new();
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", any(ws_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    let mut server = axum_server::bind_rustls(addr, config);

    server.http_builder().http2().enable_connect_protocol();

    server.serve(app.into_make_service()).await.unwrap();
}

async fn index() -> axum::response::Response {
    return (StatusCode::OK, "Hello world").into_response();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    version: Version,
    State(state): State<AppState>,
) -> axum::response::Response {
    tracing::debug!("Accepted a WebSocket using {version:?}");
    return ws.on_upgrade(|socket| handle_socket(socket, state));
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(message_result) = socket.recv().await {
        let message = match message_result {
            Ok(msg) => msg,
            Err(e) => {
                tracing::debug!("Client abruptly disconnected: {e}");
                return;
            }
        };

        let command_result = serde_json::from_str(message.to_text().unwrap());
        if let Err(e) = command_result {
            tracing::debug!("Client abruptly disconnected: {e}");
            return;
        }
        let command: Command = command_result.unwrap();
        match command.command {
            CommandType::Create => {
                let room_id = uuid::Uuid::now_v7().to_string();
                match state.rooms.lock() {
                    Ok(mut rooms) => {
                        rooms.insert(room_id.clone(), Room::new());
                    },
                    _ => {},
                };

                let response_message = format!("{{\"room_id\": \"{}\"}}", room_id);
                let send_result = socket.send(Message::from(response_message)).await;
                if let Err(e) = send_result {
                    tracing::debug!("Client abruptly disconnected: {e}")
                }
            }
            CommandType::Join => {
                let room_id = command.params.unwrap().get("room_id").unwrap().to_owned();
                let user_id = uuid::Uuid::now_v7().to_string();

                let mut result = String::new();
                match state.rooms.lock() {
                    Ok(mut rooms) => {
                        let room_option = rooms.get_mut(&room_id);
                        if let Some(room) = room_option {
                            result = room.put(user_id.clone()).unwrap();
                        }
                    },
                    _ => {},
                };

                let response_message = format!("{{\"user_id\": \"{}\", \"character\": \"{}\"}}", user_id, result);
                let send_result = socket.send(Message::from(response_message.clone())).await;
                if let Err(e) = send_result {
                    tracing::debug!("Client abruptly disconnected: {e}")
                }
            },
            CommandType::Leave => todo!(),
        }
    }
}
