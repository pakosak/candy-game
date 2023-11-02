use anyhow::Result;
use dialoguer::Input;

use crate::game::api::{CreateGameRequest, CreateGameResponse};

fn read_create_game_input() -> Result<CreateGameRequest> {
    let name: String = Input::new().with_prompt("Game name").interact_text()?;

    let width: usize = Input::new()
        .with_prompt("Width")
        .default(30)
        .interact_text()?;

    let height: usize = Input::new()
        .with_prompt("Height")
        .default(20)
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

pub async fn create_game(server: &str) -> Result<()> {
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
    println!("Created game with id: {}", resp.game_id);
    Ok(())
}
