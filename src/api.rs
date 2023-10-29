use serde::{Deserialize, Serialize};

use crate::map::Direction;

#[derive(Serialize, Debug, Deserialize)]
pub struct GameInfo {
    pub id: u64,
    pub name: String,
    pub players: Vec<String>,
    pub finished: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetGamesResponse {
    pub games: Vec<GameInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGameRequest {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub mob_cnt: usize,
    pub candy_cnt: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub game_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub game_id: u64,
    pub player_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct JoinGameResponse {
    pub player_id: u64,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PlayerAction {
    Shoot,
    Move(Direction),
}

#[derive(Deserialize)]
pub struct ActionRequest {
    pub game_id: u64,
    pub player_id: u64,
    pub action: PlayerAction,
}
