use serde::{Deserialize, Serialize};

use crate::game::map::{Direction, ObjectType, Point};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct GameInfo {
    pub id: u64,
    pub name: String,
    pub maze_name: String,
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
    pub maze_name: String,
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
    pub maze_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PlayerAction {
    Shoot,
    Move(Direction),
}

#[derive(Serialize, Deserialize)]
pub struct ActionRequest {
    pub game_id: u64,
    pub player_id: u64,
    pub action: PlayerAction,
}

#[derive(Serialize, Deserialize)]
pub struct GetStateRequest {
    pub game_id: u64,
    pub player_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetStateResponse {
    pub objects: Vec<(ObjectType, Point)>,
    pub is_finished: bool,
    pub player_winner: bool,
    pub player_dead: bool,
    pub logs: Vec<String>,
}
