use anyhow::Result;
use clap::Parser;

use game::api::*;

/// Candy game
/// Collect all candies and exit the map
/// Connect to a remote server and execute one of commands
/// Controls: arrows - move, space - shoot
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

async fn list_games(server: &str) -> Result<Vec<GameInfo>> {
    let url = format!("http://{}/games", server);
    let resp: GetGamesResponse = reqwest::get(&url).await?.json().await?;
    Ok(resp.games)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    if args.command == "list" {
        let games = list_games(&args.server).await;
        println!("Games: {:?}", games);
    } else {
        println!("Unknown command: {}", args.command);
    }
    Ok(())
}
