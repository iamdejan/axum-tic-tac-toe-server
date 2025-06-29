use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub struct Room {
    x: Option<String>,
    o: Option<String>,
    board: [[Option<char>; 3]; 3],
    current_turn: Option<char>,
    winner: Option<char>,
}

impl Room {
    pub fn new() -> Room {
        return Room {
            x: None,
            o: None,
            board: [[None; 3]; 3],
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

    pub fn has_game_started(&self) -> bool {
        return self.current_turn.is_some();
    }

    /// get_character is a function that returns the character of the user.
    pub fn get_character(&self, user_id: &String) -> Option<char> {
        let x = self.x.as_ref();
        if let Some(value) = x {
            if value == user_id {
                return Some('x');
            }
        }

        let o = self.o.as_ref();
        if let Some(value) = o {
            if value == user_id {
                return Some('o');
            }
        }

        return None;
    }

    pub fn get_current_turn(&self) -> Option<char> {
        return self.current_turn;
    }

    pub fn register_move(
        &mut self,
        row: usize,
        column: usize,
        character: char,
    ) -> Result<[[Option<char>; 3]; 3], String> {
        let square = self.board.get_mut(row).unwrap().get_mut(column).unwrap();
        if square.is_some() {
            return Err(String::from("invalid move"));
        }

        let _ = square.insert(character);
        if character == 'x' {
            self.current_turn = Some('o');
        } else {
            self.current_turn = Some('x');
        }
        return Ok(self.board.clone());
    }

    fn check_winner(&self) -> Option<char> {
        // left -> right
        for r in 0..=2 {
            let mut same_char = true;
            for c in 1..=2 {
                let prev = self.board[r][c - 1];
                let curr = self.board[r][c];
                if prev.is_none() || curr.is_none() || curr.unwrap() != prev.unwrap() {
                    same_char = false;
                    break;
                }
            }
            if same_char {
                return self.board[r][0];
            }
        }

        // top -> bottom
        for c in 0..=2 {
            let mut same_char = true;
            for r in 1..=2 {
                let prev = self.board[r - 1][c];
                let curr = self.board[r][c];
                if prev.is_none() || curr.is_none() || curr.unwrap() != prev.unwrap() {
                    same_char = false;
                    break;
                }
            }
            if same_char {
                return self.board[0][c];
            }
        }

        // diagonal: top left -> bottom right
        let mut same_char = true;
        for i in 1..=2 {
            let prev = self.board[i - 1][i - 1];
            let curr = self.board[i][i];
            if prev.is_none() || curr.is_none() || curr.unwrap() != prev.unwrap() {
                same_char = false;
                break;
            }
        }
        if same_char {
            return self.board[0][0];
        }

        // diagonal: top right -> bottom left
        let mut same_char = true;
        let mut r = 1;
        let mut c = 1;
        while r <= 2 {
            let prev = self.board[r - 1][c + 1];
            let curr = self.board[r][c];
            if prev.is_none() || curr.is_none() || curr.unwrap() != prev.unwrap() {
                same_char = false;
                break;
            }

            if c == 0 {
                break;
            }

            r += 1;
            c -= 1;
        }
        if same_char {
            return self.board[0][2];
        }

        return None;
    }

    pub fn check_and_set_winner(&mut self) -> Option<char> {
        let winner = self.check_winner();
        self.winner = winner;
        return winner;
    }

    pub fn get_user_id_from_character(&self, character: char) -> Option<(char, String)> {
        return match character {
            'x' => match self.x.clone() {
                Some(user_id) => Some(('x', user_id)),
                _ => None,
            },
            'o' => match self.o.clone() {
                Some(user_id) => Some(('o', user_id)),
                _ => None,
            },
            _ => None,
        };
    }

    pub fn has_game_finished(&self) -> bool {
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
    #[serde(alias = "move")]
    Move,
}

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub command: CommandType,
    pub params: Option<HashMap<String, String>>,
}
