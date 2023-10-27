use anyhow::Result;
use clap::Parser;
use std::io::{stdin, stdout, Write};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

mod map;
mod world;
mod world_controller;
use map::Direction;
use world::World;
use world_controller::run_world;

async fn read_player_movement(world: Arc<Mutex<World>>, player_id: u64) -> Result<()> {
    let mut keys = stdin().keys();
    let mut prev_press = Instant::now();

    while world.lock().await.win_status().is_none() {
        let key = keys.next().ok_or(anyhow::anyhow!("No key pressed"))?;

        if prev_press.elapsed() < Duration::from_millis(100) {
            continue;
        }
        let mut world = world.lock().await;

        match key? {
            Key::Esc | Key::Char('q') | Key::Ctrl('c') => {
                return Ok(());
            }
            Key::Left => world.move_player(player_id, Direction::Left),
            Key::Right => world.move_player(player_id, Direction::Right),
            Key::Up => world.move_player(player_id, Direction::Up),
            Key::Down => world.move_player(player_id, Direction::Down),
            Key::Char(' ') => world.player_shoot(player_id),
            _ => (),
        };
        prev_press = Instant::now();
    }
    Ok(())
}

async fn show_map_loop(world: Arc<Mutex<World>>, stop: Arc<AtomicBool>) -> Result<()> {
    let mut stdout = stdout().into_raw_mode()?;

    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        {
            let world = world.lock().await;

            write!(
                stdout,
                "{}{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                world.map_string().as_str()
            )?;

            let logs = world.get_logs();
            for (i, log) in logs.iter().rev().take(world.height()).enumerate() {
                write!(
                    stdout,
                    "{}{}\r\n",
                    termion::cursor::Goto(world.width() as u16 + 2, 1 + i as u16),
                    log
                )?;
            }
        }

        stdout.flush()?;

        sleep(Duration::from_millis(100)).await;
    }
    let world = world.lock().await;
    if let Some(win) = world.win_status() {
        write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto((world.width() as u16) / 2 - 3, (world.height() / 2) as u16),
            if win { "YOU WON!" } else { "YOU DIED!" },
            termion::cursor::Goto(world.width() as u16, world.height() as u16)
        )?;
    }
    Ok(())
}

/// Candy game
/// Collect all candies and exit the map
/// Controls: arrows - move, space - shoot
/// Press Esc to exit
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(verbatim_doc_comment)]
struct Args {
    /// Width of the world
    #[arg(short = 'x', default_value_t = 50)]
    width: usize,
    /// Height of the world
    #[arg(short = 'y', default_value_t = 20)]
    height: usize,
    /// Mob count
    #[arg(short = 'm', default_value_t = 10)]
    mob_cnt: usize,

    #[arg(short = 'c', default_value_t = 5)]
    candy_cnt: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    let world = Arc::new(Mutex::new(World::new(
        args.width,
        args.height,
        args.mob_cnt,
        args.candy_cnt,
    )));
    let player_id = world.lock().await.spawn_player();
    let stop = Arc::new(AtomicBool::new(false));

    run_world(world.clone());

    let world_clone = world.clone();
    let stop_clone = stop.clone();
    let show_task = tokio::spawn(async move {
        let _ = show_map_loop(world_clone, stop_clone).await;
    });

    let read_task = tokio::spawn(async move {
        let _ = read_player_movement(world, player_id).await;
    });
    read_task.await?;
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    show_task.await?;
    Ok(())
}
