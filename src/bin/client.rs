use anyhow::Result;
use clap::Parser;

use candy_game::client::create::create_game;
use candy_game::client::join::join_game;
use candy_game::client::list::list_games;

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
