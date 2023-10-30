use anyhow::{anyhow, Result};
use dialoguer::Input;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

use crate::game::api::{
    ActionRequest, GetStateRequest, GetStateResponse, JoinGameRequest, JoinGameResponse,
    PlayerAction,
};
use crate::game::map::Direction;

fn read_keystrokes(tx: Sender<PlayerAction>) -> Result<()> {
    let mut keys = stdin().keys();
    let mut prev_press = Instant::now();

    loop {
        let key = keys.next().ok_or(anyhow!("No key pressed"))?;

        if prev_press.elapsed() < Duration::from_millis(100) {
            continue;
        }

        let action: PlayerAction = match key? {
            Key::Esc | Key::Char('q') | Key::Ctrl('c') => {
                break;
            }
            Key::Left => PlayerAction::Move(Direction::Left),
            Key::Right => PlayerAction::Move(Direction::Right),
            Key::Up => PlayerAction::Move(Direction::Up),
            Key::Down => PlayerAction::Move(Direction::Down),
            Key::Char(' ') => PlayerAction::Shoot,
            _ => continue,
        };

        tx.blocking_send(action)?;

        prev_press = Instant::now();
    }
    Ok(())
}

async fn send_player_actions(
    mut rx: Receiver<PlayerAction>,
    server: &str,
    game_id: u64,
    player_id: u64,
) -> Result<()> {
    let client = reqwest::Client::new();
    loop {
        let action = rx.recv().await.ok_or(anyhow!("No msg received"))?;

        let url = format!("http://{}/action", server);
        let req = ActionRequest {
            game_id,
            player_id,
            action,
        };
        client
            .post(&url)
            .json(&req)
            .send()
            .await
            .expect("Couldn't connect to server to send action")
            .error_for_status()?;
    }
}

async fn handle_player_input(server: &str, game_id: u64, player_id: u64) -> Result<()> {
    let (tx, rx) = mpsc::channel(1);

    let blocking_read = tokio::task::spawn_blocking(move || {
        let _ = read_keystrokes(tx);
    });

    tokio::select! {
        _ = blocking_read => {},
        _ = send_player_actions(rx, server, game_id, player_id) => {},
    }

    Ok(())
}

async fn show_map_loop(
    server: &str,
    game_id: u64,
    player_id: u64,
    width: usize,
    height: usize,
) -> Result<()> {
    let mut stdout = stdout().into_raw_mode()?;

    let client = reqwest::Client::new();

    loop {
        let url = format!("http://{}/state", server);
        let req = GetStateRequest { game_id, player_id };

        let state: GetStateResponse = client
            .post(&url)
            .json(&req)
            .send()
            .await
            .expect("Couldn't connect to server to get state")
            .json()
            .await?;

        write!(
            stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            state.map
        )?;

        for (i, log) in state.logs.iter().rev().take(height).enumerate() {
            write!(
                stdout,
                "{}{}\r\n",
                termion::cursor::Goto(width as u16 + 2, 1 + i as u16),
                log
            )?;
        }
        write!(
            stdout,
            "{}",
            termion::cursor::Goto(width as u16, height as u16),
        )?;

        stdout.flush()?;

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn join_game(server: &str) -> Result<()> {
    let url = format!("http://{}/join", server);

    let game_id: u64 = Input::new()
        .with_prompt("Game ID")
        .default(0)
        .interact_text()?;
    let player_name: String = Input::new().with_prompt("Player name").interact_text()?;

    let req = JoinGameRequest {
        game_id,
        player_name,
    };

    let resp = reqwest::Client::new()
        .post(&url)
        .json(&req)
        .send()
        .await
        .expect("Couldn't connect to server to join game");

    if resp.status().is_client_error() {
        println!("Error joining game: {}", resp.text().await?);
        return Ok(());
    }
    let resp: JoinGameResponse = resp.json().await?;
    println!("Joined with player id: {}", resp.player_id);

    tokio::select! {
        _ = show_map_loop(server, game_id, resp.player_id, resp.width, resp.height) => {},
        _ = handle_player_input(server, game_id, resp.player_id) => {},
    };

    Ok(())
}
