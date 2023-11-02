use anyhow::Result;

use axum::extract::State;
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
use tokio::time::Instant;

use candy_game::game::api::*;
use candy_game::game::world::World;
use candy_game::game::world_controller::run_world;

const CLIENT_MAX_PING_S: u64 = 5;

struct Game {
    name: String,
    players: HashMap<u64, String>,
    players_last_seen: HashMap<u64, Instant>,
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
            finished: game.world.lock().await.get_state().winner.is_some(),
        });
    }
    Json(resp)
}

async fn create_game(
    State(games): State<SharedGames>,
    Json(req): Json<CreateGameRequest>,
) -> Json<CreateGameResponse> {
    let mut games = games.lock().await;
    let game_id = games.len() as u64;
    games.insert(
        game_id,
        Game {
            name: req.name.clone(),
            players: HashMap::new(),
            players_last_seen: HashMap::new(),
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
    Json(CreateGameResponse { game_id })
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
        let mut world = game.world.lock().await;
        let player_id = world.spawn_player(req.player_name.as_str());
        game.players.insert(player_id, req.player_name.clone());
        info!(
            "Player {} with id {} joined game {}",
            req.player_name, player_id, req.game_id
        );
        (
            StatusCode::OK,
            Json(JoinGameResponse {
                player_id,
                width: world.width(),
                height: world.height(),
                map: world.get_map_template().format(),
            }),
        )
            .into_response()
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
    Json(req): Json<GetStateRequest>,
) -> impl IntoResponse {
    if let Some(game) = games.lock().await.get_mut(&req.game_id) {
        game.players_last_seen.insert(req.player_id, Instant::now());

        let world = game.world.lock().await;
        let state = world.get_state();
        let resp = GetStateResponse {
            objects: state.objects,
            is_finished: state.winner.is_some(),
            player_winner: state.winner.is_some_and(|winner| winner == req.player_id),
            player_dead: state.dead_players.contains(&req.player_id),
            logs: state.logs.clone(),
        };
        (StatusCode::OK, Json(resp)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Game {} not found", req.game_id),
        )
            .into_response()
    }
}

async fn do_action(
    State(games): State<SharedGames>,
    Json(req): Json<ActionRequest>,
) -> impl IntoResponse {
    if let Some(game) = games.lock().await.get_mut(&req.game_id) {
        let mut world = game.world.lock().await;
        let state = world.get_state();
        if state.winner.is_some() {
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

async fn clean_idle_players(games: SharedGames) {
    loop {
        let mut games = games.lock().await;
        for game in games.values_mut() {
            let idle_players: Vec<u64> = game
                .players_last_seen
                .iter()
                .filter_map(|(id, last_seen)| {
                    if last_seen.elapsed().as_secs() > CLIENT_MAX_PING_S {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

            for player_id in idle_players {
                info!(
                    "Player {} ({}) removed from game {}",
                    player_id, game.players[&player_id], game.name
                );
                game.world.lock().await.erase_player(player_id);
                game.players_last_seen.remove(&player_id);
                game.players.remove(&player_id);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let games: SharedGames = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(clean_idle_players(games.clone()));

    let app = Router::new()
        .route("/games", get(list_games))
        .route("/create", post(create_game))
        .route("/join", post(join_game))
        .route("/action", post(do_action))
        .route("/state", post(game_state))
        .with_state(games);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    info!("Starting server at {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
