use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub struct Room {
    pub x: Option<String>,
    pub o: Option<String>,
    pub _board: [[Option<char>; 3]; 3],
    pub _current_turn: Option<String>,
    pub _winner: Option<String>,
}

impl Room {
    pub fn new() -> Room {
        return Room {
            x: Option::None,
            o: Option::None,
            _board: [[Option::None; 3]; 3],
            _current_turn: Option::None,
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
}

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
}

impl AppState {
    pub fn new() -> AppState {
        return AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
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
