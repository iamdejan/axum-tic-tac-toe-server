use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub struct Room {
    x: Option<String>,
    o: Option<String>,
    _board: [[Option<char>; 3]; 3],
    current_turn: Option<char>,
    _winner: Option<String>,
}

impl Room {
    pub fn new() -> Room {
        return Room {
            x: Option::None,
            o: Option::None,
            _board: [[Option::None; 3]; 3],
            current_turn: Option::None,
            _winner: Option::None,
        };
    }

    pub fn put(&mut self, user_id: String) -> Result<char, String> {
        if self.x.is_none() {
            self.x = Option::Some(user_id);
            return Ok('x');
        }
        if self.o.is_none() {
            self.o = Option::Some(user_id);
            return Ok('o');
        }

        return Err(String::from("Room is already full"));
    }

    pub fn is_full(&self) -> bool {
        return self.x.is_some() && self.o.is_some();
    }

    pub fn start_game(&mut self) {
        self.current_turn = Option::Some('x');
    }

    pub fn game_is_started(&self) -> bool {
        return self.current_turn.is_some();
    }
}

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub tx: tokio::sync::broadcast::Sender<String>
}

impl AppState {
    pub fn new() -> AppState {
        return AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            tx: tokio::sync::broadcast::channel(100).0,
        };
    }
}

#[derive(Serialize, Deserialize)]
pub enum CommandType {
    #[serde(alias = "create")]
    Create,
    #[serde(alias = "join")]
    Join,
    #[serde(alias = "leave")]
    Leave,
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub command: CommandType,
    pub params: Option<HashMap<String, String>>,
}
