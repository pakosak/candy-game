use anyhow::Result;
use prettytable::{row, Cell, Row, Table};

use crate::game::api::GetGamesResponse;

pub async fn list_games(server: &str) -> Result<()> {
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
