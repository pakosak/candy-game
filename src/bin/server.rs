use anyhow::Result;

use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log::info;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use game::api::*;
use game::world::World;
use game::world_controller::run_world;

struct Game {
    name: String,
    players: HashMap<u64, String>,
    world: Arc<Mutex<World>>,
}

type SharedGames = Arc<Mutex<HashMap<u64, Game>>>;

async fn list_games(State(games): State<SharedGames>) -> Json<GetGamesResponse> {
    let mut resp = GetGamesResponse { games: Vec::new() };
    for (id, game) in games.lock().await.iter() {
        resp.games.push(GameInfo {
            id: *id,
            name: game.name.clone(),
            players: game.players.values().cloned().collect::<Vec<String>>(),
            finished: game.world.lock().await.get_state().finished,
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
            players: HashMap::new(),
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
    todo!("return game id");
    StatusCode::OK
}

async fn join_game(
    State(games): State<SharedGames>,
    Json(req): Json<JoinGameRequest>,
) -> impl IntoResponse {
    let mut games = games.lock().await;
    if let Some(game) = games.get_mut(&req.game_id) {
        if game.players.values().any(|val| val == &req.player_name) {
            return (
                StatusCode::BAD_REQUEST,
                format!("Player {} already in game {}", req.player_name, req.game_id),
            )
                .into_response();
        }
        let player_id = game.world.lock().await.spawn_player();
        game.players.insert(player_id, req.player_name.clone());
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

async fn game_state(
    State(games): State<SharedGames>,
    Path(game_id): Path<u64>,
) -> impl IntoResponse {
    if let Some(game) = games.lock().await.get(&game_id) {
        (StatusCode::OK, Json(game.world.lock().await.get_state())).into_response()
    } else {
        (StatusCode::NOT_FOUND, format!("Game {} not found", game_id)).into_response()
    }
}

async fn do_action(
    State(games): State<SharedGames>,
    Json(req): Json<ActionRequest>,
) -> impl IntoResponse {
    if let Some(game) = games.lock().await.get_mut(&req.game_id) {
        let mut world = game.world.lock().await;
        let state = world.get_state();
        if state.finished {
            return (
                StatusCode::BAD_REQUEST,
                format!("Game {} already finished", req.game_id),
            )
                .into_response();
        }
        if state.dead_players.contains(&req.player_id) {
            return (
                StatusCode::BAD_REQUEST,
                format!("Player {} already dead", req.player_id),
            )
                .into_response();
        }
        if !game.players.contains_key(&req.player_id) {
            return (
                StatusCode::BAD_REQUEST,
                format!("Player {} not in game {}", req.player_id, req.game_id),
            )
                .into_response();
        }
        match req.action {
            PlayerAction::Shoot => world.player_shoot(req.player_id),
            PlayerAction::Move(dir) => world.move_player(req.player_id, dir),
        }
        (StatusCode::OK, "OK").into_response()
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
        .route("/state/:game_id", get(game_state))
        .route("/games", get(list_games))
        .route("/create", post(create_game))
        .route("/join", post(join_game))
        .route("/action", post(do_action))
        .with_state(games);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    info!("Starting server at {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
