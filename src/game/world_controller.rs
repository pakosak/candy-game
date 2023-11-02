use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::game::world::World;

pub fn run_world(world: Arc<Mutex<World>>) {
    tokio::spawn(async move {
        loop {
            world.lock().await.move_world();
            sleep(Duration::from_millis(100)).await;
        }
    });
}
