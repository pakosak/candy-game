use clap::Parser;
use std::io::{stdin, stdout, Write};
use std::time::Instant;
use std::{
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod map;
mod world;
use map::Direction;
use world::World;

fn read_player_movement(world: Arc<Mutex<World>>) {
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut keys = stdin.keys();
    let mut prev_press = Instant::now();

    while world.lock().unwrap().win_status().is_none() {
        let key = keys.next().unwrap().unwrap();

        if prev_press.elapsed() < Duration::from_millis(100) {
            continue;
        }
        let mut world = world.lock().unwrap();

        match key {
            Key::Esc => {
                return;
            }
            Key::Left => world.move_player(Direction::Left),
            Key::Right => world.move_player(Direction::Right),
            Key::Up => world.move_player(Direction::Up),
            Key::Down => world.move_player(Direction::Down),
            Key::Char(' ') => world.player_shoot(),
            _ => (),
        };
        prev_press = Instant::now();
    }
}

fn move_mobs_loop(world: Arc<Mutex<World>>) {
    loop {
        world.lock().unwrap().move_random_mob();
        sleep(Duration::from_millis(100));
    }
}

fn move_shots_loop(world: Arc<Mutex<World>>) {
    loop {
        world.lock().unwrap().move_shots();
        sleep(Duration::from_millis(50));
    }
}

fn show_map_loop(world: Arc<Mutex<World>>, stop: Arc<Mutex<bool>>) {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    while !*stop.lock().unwrap() {
        {
            let world = world.lock().unwrap();

            write!(
                stdout,
                "{}{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                world.map_string().as_str()
            )
            .unwrap();

            let logs = world.get_logs();
            for (i, log) in logs.iter().rev().take(world.height()).enumerate() {
                write!(
                    stdout,
                    "{}{}\r\n",
                    termion::cursor::Goto(world.width() as u16 + 2, 1 + i as u16),
                    log
                )
                .unwrap();
            }
        }

        stdout.flush().unwrap();

        sleep(Duration::from_millis(100));
    }
    let world = world.lock().unwrap();
    if let Some(win) = world.win_status() {
        write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto((world.width() as u16) / 2 - 3, (world.height() / 2) as u16),
            if win { "YOU WON!" } else { "YOU DIED!" },
            termion::cursor::Goto(world.width() as u16, world.height() as u16)
        )
        .unwrap();
    }
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

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let world = Arc::new(Mutex::new(World::new(
        args.width,
        args.height,
        args.mob_cnt,
        args.candy_cnt,
    )));
    let stop = Arc::new(Mutex::new(false));

    {
        let world = world.clone();
        thread::spawn(move || move_mobs_loop(world));
    }
    {
        let world = world.clone();
        thread::spawn(move || move_shots_loop(world));
    }

    let world_clone = world.clone();
    let stop_clone = stop.clone();
    let show_handle = thread::spawn(move || show_map_loop(world_clone, stop_clone));

    let read_handle = thread::spawn(move || read_player_movement(world));
    read_handle.join().unwrap();
    *stop.lock().unwrap() = true;
    show_handle.join().unwrap();
}
