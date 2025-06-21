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
    winner: Option<String>,
}

impl Room {
    pub fn new() -> Room {
        return Room {
            x: None,
            o: None,
            _board: [[None; 3]; 3],
            current_turn: None,
            winner: None,
        };
    }

    pub fn join(&mut self, user_id: String) -> Result<char, String> {
        match self.x.clone() {
            None => {
                self.x = Some(user_id);
                return Ok('x');
            }
            Some(assigned_user_id) => {
                if assigned_user_id == user_id {
                    return Ok('x');
                }
            }
        }

        match self.o.clone() {
            None => {
                self.o = Some(user_id);
                return Ok('o');
            }
            Some(assigned_user_id) => {
                if assigned_user_id == user_id {
                    return Ok('o');
                }
            }
        }

        return Err(String::from("Room is already full"));
    }

    pub fn leave(&mut self, user_id: String) -> Result<char, String> {
        let e = Err(String::from("User never joined this room"));

        match self.x.clone() {
            Some(assigned_user_id) => {
                if assigned_user_id == user_id {
                    self.x = None;
                    return Ok('x');
                }
            }
            _ => {}
        }

        match self.o.clone() {
            Some(assigned_user_id) => {
                if assigned_user_id == user_id {
                    self.o = None;
                    return Ok('o');
                } else {
                    return e;
                }
            }
            None => {
                return e;
            }
        }
    }

    pub fn is_full(&self) -> bool {
        return self.x.is_some() && self.o.is_some();
    }

    pub fn start_game(&mut self) {
        self.current_turn = Option::Some('x');
    }

    pub fn is_game_started(&self) -> bool {
        return self.current_turn.is_some();
    }

    pub fn is_game_ended(&self) -> bool {
        return self.winner.is_some();
    }
}

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub sender: tokio::sync::broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> AppState {
        return AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            sender: tokio::sync::broadcast::channel(100).0,
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
