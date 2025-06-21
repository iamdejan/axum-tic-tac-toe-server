use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub struct Room {
    _x: Option<String>,
    _o: Option<String>,
    _board: [[Option<char>; 3]; 3],
    _current_turn: Option<char>,
    _winner: Option<String>,
}

impl Room {
    pub fn new() -> Room {
        return Room {
            _x: Option::None,
            _o: Option::None,
            _board: [[Option::None; 3]; 3],
            _current_turn: Option::None,
            _winner: Option::None,
        };
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
