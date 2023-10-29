use anyhow::Result;
use dialoguer::Input;

use crate::game::api::{JoinGameRequest, JoinGameResponse};

pub async fn join_game(server: &str) -> Result<()> {
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
