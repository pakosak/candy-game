use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::game::world::World;

pub fn run_world(world: Arc<Mutex<World>>) {
    {
        let world = world.clone();
        tokio::spawn(async move {
            loop {
                world.lock().await.move_random_mob();
                sleep(Duration::from_millis(100)).await;
            }
        });
    }
    {
        tokio::spawn(async move {
            loop {
                world.lock().await.move_shots();
                sleep(Duration::from_millis(50)).await;
            }
        });
    }
}
