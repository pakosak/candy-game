use anyhow::Result;
use clap::Parser;
use dialoguer::Select;
use std::io::{stdout, Write};

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
    #[arg(short = 's', default_value_t = String::from("localhost:3030"))]
    server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    const COMMANDS: [&str; 4] = ["list", "create", "join", "quit"];
    let mut stdout = stdout();

    loop {
        let command: usize = Select::new()
            .with_prompt("Game menu")
            .items(&COMMANDS)
            .default(0)
            .interact()?;

        if command == 0 {
            list_games(&args.server).await?;
        } else if command == 1 {
            create_game(&args.server).await?;
        } else if command == 2 {
            join_game(&args.server).await?;
        } else {
            break;
        }

        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        )?;
    }

    Ok(())
}
