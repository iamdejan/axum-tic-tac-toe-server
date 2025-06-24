use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::Message},
    http::{StatusCode, Version},
    response::IntoResponse,
    routing::{any, get},
};
use axum_server::tls_rustls::RustlsConfig;
use serde_json::json;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod game;
use crate::game::{AppState, CommandType, Room, WebSocketMessage};

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
    let mut receiver = state.sender.subscribe();
    return ws.on_upgrade(|mut socket| async move {
        loop {
            tokio::select! {
                res = socket.recv() => {
                    match res {
                        Some(message_result) => {
                            if let Err(e) = message_result {
                                tracing::warn!("Error on receiving message from socket: {e}");
                                continue
                            }

                            handle_socket_recv(&state, message_result.unwrap());
                        },
                        _ => {},
                    }
                }
                res = receiver.recv() => {
                    match res {
                        Ok(msg) => {
                            if let Err(e) = socket.send(Message::from(msg)).await {
                                tracing::warn!("Error on receiving message from state's sender: {e}");
                                continue
                            }
                        },
                        _ => continue
                    }
                }
            }
        }
    });
}

fn handle_socket_recv(state: &AppState, message: Message) {
    let ws_message_result = serde_json::from_str::<WebSocketMessage>(message.to_text().unwrap());
    if let Err(e) = ws_message_result {
        tracing::warn!("Fail to parse message: {e}");
        return;
    }
    let ws_message = ws_message_result.unwrap();
    match ws_message.command {
        CommandType::Create => {
            create_room(state);
        }
        CommandType::Join => {
            let params = ws_message.params.unwrap();
            join_room(state, params);
        }
        CommandType::Leave => {
            let params = ws_message.params.unwrap();
            leave_room(state, params);
        }
        CommandType::Move => {
            let params = ws_message.params.unwrap();
            register_move(state, params);
        }
    }
}

fn create_room(state: &AppState) {
    let room_id = uuid::Uuid::now_v7().to_string();
    match state.rooms.lock() {
        Ok(mut rooms) => {
            rooms.insert(room_id.clone(), Room::new());
        }
        Err(e) => {
            tracing::error!("Fail to lock room: {e}");
            return;
        }
    };

    let message = json!({
        "room_id": room_id
    });
    let send_result = state.sender.send(message.to_string());
    if let Err(e) = send_result {
        tracing::warn!("Send message failed: {e}");
        return;
    }
}

fn join_room(state: &AppState, params: HashMap<String, String>) {
    let room_id = params.get("room_id").unwrap().to_string();
    let user_id = params.get("user_id").unwrap().to_string();

     let is_game_started = has_game_started(state, &room_id);
     if is_game_started {
        let message = json!({
            "room_id": &room_id,
            "error": "Game has already started!",
            "user_id": &user_id
        });
        state.sender.send(message.to_string()).unwrap();
        return;
    }

    let is_full = is_room_full(&state, &room_id);
    if is_full {
        let message = json!({
            "room_id": &room_id,
            "user_id": user_id,
            "error": "Room is already full"
        });
        state.sender.send(message.to_string()).unwrap();
        return;
    }

    let character_result =
        get_room_and_execute_result(state, &room_id, |room| room.join(user_id.clone()));
    let send_result = match character_result {
        Ok(character) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": user_id,
                "event": "ROOM_JOINED",
                "character": character,
            });
            state.sender.send(message.to_string())
        }
        Err(e) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": user_id,
                "error": e,
            });
            state.sender.send(message.to_string())
        }
    };
    if let Err(e) = send_result {
        tracing::warn!("Send message failed: {e}");
        return;
    }

    if is_room_full(&state, &room_id) {
        let message = json!({
            "room_id": &room_id,
            "event": "GAME_STARTED"
        });
        state.sender.send(message.to_string()).unwrap();
    }
}

fn leave_room(state: &AppState, params: HashMap<String, String>) {
    let room_id = params.get("room_id").unwrap().to_string();
    let user_id = params.get("user_id").unwrap().to_string();

    let is_game_started = has_game_started(state, &room_id);
    if is_game_started {
        let message = json!({
            "room_id": &room_id,
            "error": "Game has already started!",
            "user_id": &user_id
        });
        state.sender.send(message.to_string()).unwrap();
        return;
    }

    let leave_result =
        get_room_and_execute_result(state, &room_id, |room| room.leave(user_id.clone()));
    let send_result = match leave_result {
        Ok(prev_char) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": &user_id,
                "event": "ROOM_LEFT",
                "character": prev_char,
            });
            state.sender.send(message.to_string())
        }
        Err(e) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": user_id,
                "error": e,
            });
            state.sender.send(message.to_string())
        }
    };
    if let Err(e) = send_result {
        tracing::warn!("Send message failed: {e}");
    }
}

fn register_move(state: &AppState, params: HashMap<String, String>) {
    let room_id = params.get("room_id").unwrap().to_string();
    let user_id = params.get("user_id").unwrap().to_string();

    let has_game_finished = has_game_finished(state, &room_id);
    if has_game_finished {
        let message = json!({
            "room_id": &room_id,
            "error": "Game has already finished!",
            "user_id": &user_id
        });
        state.sender.send(message.to_string()).unwrap();
        return;
    }

    let row = params
        .get("row")
        .unwrap()
        .to_string()
        .parse::<usize>()
        .unwrap();
    let column = params
        .get("column")
        .unwrap()
        .to_string()
        .parse::<usize>()
        .unwrap();

    let character = get_room_and_execute_result(state, &room_id, |room| {
        let result = room.get_character(&user_id);
        return match result {
            Some(value) => Ok(value),
            None => Err(String::from("User not found")),
        };
    })
    .unwrap();
    let register_move_result = get_room_and_execute_result(state, &room_id, |room| {
        room.register_move(row, column, character)
    });
    let send_result = match register_move_result {
        Ok(board) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": &user_id,
                "event": "MOVE_REGISTERED",
                "board_after_move": board
            });
            state.sender.send(message.to_string())
        }
        Err(e) => {
            let message = json!({
                "room_id": &room_id,
                "user_id": user_id,
                "error": e,
            });
            state.sender.send(message.to_string())
        }
    };
    if let Err(e) = send_result {
        tracing::warn!("Send message failed: {e}");
    }

    let winner_user_result = get_room_and_execute_option(state, &room_id, |room| {
        let w = room.check_and_set_winner();
        match w {
            Some(character) => room.get_user_id_from_character(character),
            _ => None,
        }
    });
    if let Some(winner_user) = winner_user_result {
        let message = json!({
            "room_id": &room_id,
            "user_id": &user_id,
            "event": "GAME_FINISHED",
            "winner_user_id": winner_user.1,
            "winner_character": winner_user.0,
        });
        state.sender.send(message.to_string()).unwrap();
    }
}

fn is_room_full(state: &AppState, room_id: &String) -> bool {
    let result = get_room_and_execute_result(state, room_id, |room| {
        if room.is_full() && !room.has_game_started() && !room.has_game_finished() {
            room.start_game();
        }

        return Ok(room.is_full());
    });
    return match result {
        Ok(b) => b,
        Err(_) => false,
    };
}

fn has_game_started(state: &AppState, room_id: &String) -> bool {
    let result = get_room_and_execute_result(state, room_id, |room| Ok(room.has_game_started()));
    return match result {
        Ok(b) => b,
        Err(_) => false,
    };
}

fn has_game_finished(state: &AppState, room_id: &String) -> bool {
    let result = get_room_and_execute_result(state, room_id, |room| Ok(room.has_game_finished()));
    return match result {
        Ok(b) => b,
        Err(_) => false,
    };
}

/// references:
/// - https://www.reddit.com/r/learnrust/comments/xvxpy2/is_there_a_workaround_for_variable_capturing_in/
/// - https://doc.rust-lang.org/book/ch13-01-closures.html
fn get_room_and_execute_result<T, F>(state: &AppState, room_id: &String, f: F) -> Result<T, String>
where
    F: FnOnce(&mut Room) -> Result<T, String>,
{
    return match state.rooms.lock() {
        Ok(mut rooms) => match rooms.get_mut(room_id) {
            Some(room) => f(room),
            None => Err(String::from("Room not found")),
        },
        Err(e) => Err(format!("Fail to lock room: {}", e).to_string()),
    };
}

/// references:
/// - https://www.reddit.com/r/learnrust/comments/xvxpy2/is_there_a_workaround_for_variable_capturing_in/
/// - https://doc.rust-lang.org/book/ch13-01-closures.html
fn get_room_and_execute_option<T, F>(state: &AppState, room_id: &String, f: F) -> Option<T>
where
    F: FnOnce(&mut Room) -> Option<T>,
{
    return match state.rooms.lock() {
        Ok(mut rooms) => match rooms.get_mut(room_id) {
            Some(room) => f(room),
            None => None,
        },
        Err(_) => None,
    };
}
