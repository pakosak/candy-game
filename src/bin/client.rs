use anyhow::Result;
use clap::Parser;
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

async fn create_game(server: &str) -> Result<()> {
    let url = format!("http://{}/create", server);
    let req = CreateGameRequest {
        name: "My game".to_string(),
        width: 50,
        height: 20,
        mob_cnt: 10,
        candy_cnt: 5,
    };
    let resp: CreateGameResponse = reqwest::Client::new()
        .post(&url)
        .json(&req)
        .send()
        .await?
        .json()
        .await?;
    println!("Game created: {}", resp.game_id);
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
    } else {
        println!("Unknown command: {}", args.command);
    }
    Ok(())
}
