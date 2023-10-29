use anyhow::Result;
use clap::Parser;
use dialoguer::Input;
use prettytable::{row, Cell, Row, Table};

use game::api::*;

/// Candy game
/// Collect all candies and exit the map
/// Connect to a remote server and execute one of commands
/// Controls after joining: arrows - move, space - shoot
/// Press Esc/q to exit
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(verbatim_doc_comment)]
struct Args {
    /// Server address
    #[arg(short = 's')]
    server: String,
    /// Command to execute (list, create, join)
    command: String,
}

async fn list_games(server: &str) -> Result<()> {
    let url = format!("http://{}/games", server);
    let resp: GetGamesResponse = reqwest::get(&url).await?.json().await?;

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Name"),
        Cell::new("Players"),
        Cell::new("Finished"),
    ]));
    for game in resp.games {
        table.add_row(row!(
            &game.id.to_string(),
            &game.name,
            &game.players.join(", "),
            if game.finished { "Yes" } else { "No" }
        ));
    }
    table.printstd();

    Ok(())
}

fn read_create_game_input() -> Result<CreateGameRequest> {
    let name: String = Input::new().with_prompt("Game name").interact_text()?;

    let width: usize = Input::new()
        .with_prompt("Width")
        .default(50)
        .interact_text()?;

    let height: usize = Input::new()
        .with_prompt("Height")
        .default(50)
        .interact_text()?;

    let mob_cnt: usize = Input::new()
        .with_prompt("Mob count")
        .default(10)
        .interact_text()?;

    let candy_cnt: usize = Input::new()
        .with_prompt("Candy count")
        .default(5)
        .interact_text()?;

    Ok(CreateGameRequest {
        name,
        width,
        height,
        mob_cnt,
        candy_cnt,
    })
}

async fn create_game(server: &str) -> Result<()> {
    let url = format!("http://{}/create", server);

    let req = read_create_game_input()?;

    let resp: CreateGameResponse = reqwest::Client::new()
        .post(&url)
        .json(&req)
        .send()
        .await
        .expect("Couldn't connect to server to create game")
        .json()
        .await?;
    println!("Game created: {}", resp.game_id);
    Ok(())
}

async fn join_game(server: &str) -> Result<()> {
    let url = format!("http://{}/join", server);

    let game_id: u64 = Input::new().with_prompt("Game ID").interact_text()?;
    let player_name: String = Input::new().with_prompt("Player name").interact_text()?;

    let req = JoinGameRequest {
        game_id,
        player_name,
    };

    let resp: JoinGameResponse = reqwest::Client::new()
        .post(&url)
        .json(&req)
        .send()
        .await
        .expect("Couldn't connect to server to join game")
        .error_for_status()?
        .json()
        .await?;
    println!("Joined with player id: {}", resp.player_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    if args.command == "list" {
        list_games(&args.server).await?;
    } else if args.command == "create" {
        create_game(&args.server).await?;
    } else if args.command == "join" {
        join_game(&args.server).await?;
    } else {
        println!("Unknown command: {}", args.command);
    }
    Ok(())
}
