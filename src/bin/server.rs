use anyhow::Result;

use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    response::{self, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use env_logger;
use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use game::world::World;
use game::world_controller::run_world;

#[derive(serde::Serialize)]
struct GameInfo {
    id: u64,
    name: String,
    players: Vec<String>,
    finished: bool,
}

#[derive(serde::Serialize)]
struct GetGamesResponse {
    games: Vec<GameInfo>,
}

#[derive(serde::Deserialize, Debug)]
struct CreateGameRequest {
    name: String,
    width: usize,
    height: usize,
    mob_cnt: usize,
    candy_cnt: usize,
}

#[derive(serde::Deserialize)]
struct JoinGameRequest {
    game_id: u64,
    player_name: String,
}

#[derive(serde::Serialize)]
struct JoinGameResponse {
    player_id: u64,
}

struct Game {
    name: String,
    players: Vec<String>,
    world: Arc<Mutex<World>>,
}

type SharedGames = Arc<Mutex<HashMap<u64, Game>>>;

async fn list_games(State(games): State<SharedGames>) -> Json<GetGamesResponse> {
    let mut resp = GetGamesResponse { games: Vec::new() };
    for (id, game) in games.lock().await.iter() {
        resp.games.push(GameInfo {
            id: *id,
            name: game.name.clone(),
            players: game.players.clone(),
            finished: game.world.lock().await.win_status().is_some(),
        });
    }
    Json(resp)
}

async fn create_game(
    State(games): State<SharedGames>,
    Json(req): Json<CreateGameRequest>,
) -> StatusCode {
    let mut games = games.lock().await;
    let game_id = games.len() as u64;
    games.insert(
        game_id,
        Game {
            name: req.name.clone(),
            players: Vec::new(),
            world: Arc::new(Mutex::new(World::new(
                req.width,
                req.height,
                req.mob_cnt,
                req.candy_cnt,
            ))),
        },
    );
    run_world(Arc::clone(&games.get(&game_id).unwrap().world));
    info!("Game {} created using {:?}", game_id, req);
    StatusCode::OK
}

async fn join_game(
    State(games): State<SharedGames>,
    Json(req): Json<JoinGameRequest>,
) -> impl IntoResponse {
    let mut games = games.lock().await;
    if let Some(game) = games.get_mut(&req.game_id) {
        if game.players.contains(&req.player_name) {
            return (
                StatusCode::BAD_REQUEST,
                format!("Player {} already in game {}", req.player_name, req.game_id),
            )
                .into_response();
        }
        let player_id = game.world.lock().await.spawn_player();
        game.players.push(req.player_name.clone());
        info!("Player {} joined game {}", req.player_name, req.game_id);
        (StatusCode::OK, Json(JoinGameResponse { player_id })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Game {} not found", req.game_id),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let games: SharedGames = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        // .route("/state/:game_id", get(game_state))
        .route("/games", get(list_games))
        .route("/create", post(create_game))
        .route("/join", post(join_game))
        .with_state(games);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    info!("Starting server at {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
