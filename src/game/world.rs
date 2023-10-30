use chrono::Local;
use std::collections::HashMap;

use crate::game::map::{Direction, Map, ObjectType, OrientedPoint, Point};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

impl Point {
    fn step(&self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
    fn update(&mut self, new_pos: Point) {
        self.x = new_pos.x;
        self.y = new_pos.y;
    }
}

#[derive(serde::Serialize)]
pub struct WorldState<'a> {
    pub map: String,
    pub finished: bool,
    pub dead_players: Vec<u64>,
    pub logs: &'a Vec<String>,
}

#[derive(Default)]
pub struct World {
    map: Map,
    players: HashMap<u64, OrientedPoint>,
    exit: Point,
    mobs: HashMap<u64, Point>,
    shots: HashMap<u64, OrientedPoint>,

    finished: bool,
    dead_players: Vec<u64>,
    player_names: HashMap<u64, String>,
    candies_left: usize,
    logs: Vec<String>,
}

impl World {
    pub fn new(width: usize, height: usize, mob_cnt: usize, candy_cnt: usize) -> Self {
        let map_ = Map::new(width, height);
        let mut w = World {
            map: map_,
            candies_left: candy_cnt,
            ..Default::default()
        };
        w.exit = w.map.random_empty_point();
        w.map.place_object(ObjectType::Exit, &w.exit);
        w.log(format!("Exit placed at {:?}", w.exit));

        for _ in 0..mob_cnt {
            let mob = w.map.random_empty_point();
            let mob_id = rand::random();
            w.map.place_object_with_id(mob_id, ObjectType::Mob, &mob);
            w.mobs.insert(mob_id, mob);
        }
        w.log(format!("{} mobs placed", mob_cnt));
        for _ in 0..candy_cnt {
            let candy = w.map.random_empty_point();
            w.map.place_object(ObjectType::Candy, &candy);
        }
        w.log(format!("{} candies placed", candy_cnt));
        w
    }

    pub fn can_play(&self, player_id: u64) -> bool {
        !self.finished && !self.dead_players.contains(&player_id)
    }

    pub fn get_state(&self) -> WorldState {
        WorldState {
            map: self.map.format(),
            finished: self.finished,
            dead_players: self.dead_players.clone(),
            logs: &self.logs,
        }
    }

    pub fn width(&self) -> usize {
        self.map.width()
    }

    pub fn height(&self) -> usize {
        self.map.height()
    }

    fn log(&mut self, msg: String) {
        self.logs
            .push(format!("{}: {}", Local::now().format("%H:%M:%S"), msg));
    }

    pub fn move_random_mob(&mut self) {
        let mob_cnt = self.mobs.len();
        if mob_cnt == 0 {
            return;
        }
        if let Some((_, mob_pos)) = self.mobs.iter_mut().nth(rand::random::<usize>() % mob_cnt) {
            let new_pos = mob_pos.step(rand::random());
            let collided_obj = self.map.get_object(&new_pos);
            match collided_obj.type_ {
                ObjectType::Empty => {
                    self.map.swap_objects(mob_pos, &new_pos);
                    mob_pos.update(new_pos);
                }
                ObjectType::Player(_) => {
                    self.players.remove(&collided_obj.id);
                    self.dead_players.push(collided_obj.id);
                    self.map.swap_objects(mob_pos, &new_pos);
                    self.map.clear_object(mob_pos);
                    mob_pos.update(new_pos);
                }
                _ => (),
            }
        }
    }

    pub fn move_shots(&mut self) {
        let mut new_logs: Vec<String> = Vec::new();
        self.shots.retain(|_, shot| {
            let new_pos = shot.point.step(shot.dir);
            let collider_obj = self.map.get_object(&new_pos);
            match collider_obj.type_ {
                ObjectType::Empty => {
                    self.map.swap_objects(&shot.point, &new_pos);
                    shot.point.update(new_pos);
                    true
                }
                ObjectType::Mob => {
                    new_logs.push("Mob killed by stray shot".to_string());
                    self.mobs.remove(&collider_obj.id);
                    self.map.clear_object(&shot.point);
                    self.map.clear_object(&new_pos);
                    false
                }
                ObjectType::Player(_) => {
                    new_logs.push(format!(
                        "{} killed by stray shot",
                        self.player_names[&collider_obj.id]
                    ));
                    self.players.remove(&collider_obj.id);
                    self.dead_players.push(collider_obj.id);
                    self.map.clear_object(&shot.point);
                    self.map.clear_object(&new_pos);
                    false
                }
                _ => {
                    self.map.clear_object(&shot.point);
                    false
                }
            }
        });
        for log in new_logs {
            self.log(log);
        }
    }

    pub fn spawn_player(&mut self, player_name: &str) -> u64 {
        let player = OrientedPoint {
            point: self.map.random_empty_point(),
            dir: Direction::Right,
        };
        let player_id = rand::random();
        self.map.place_object_with_id(
            player_id,
            ObjectType::Player(Direction::Right),
            &player.point,
        );
        self.players.insert(player_id, player);
        self.player_names.insert(player_id, player_name.to_string());
        self.log(format!("Player {} entered world", player_name));
        player_id
    }

    pub fn move_player(&mut self, player_id: u64, direction: Direction) {
        let mut player = self
            .players
            .remove(&player_id)
            .unwrap_or_else(|| panic!("Player {} not found", player_id));
        player.dir = direction;
        let new_pos = player.point.step(direction);
        let collider_obj = self.map.get_object(&new_pos);
        match collider_obj.type_ {
            ObjectType::Empty => {
                self.map.clear_object(&player.point);
                player.point = new_pos;
            }
            ObjectType::Exit => {
                if self.candies_left != 0 {
                    self.log(format!(
                        "You need to collect {} more candies",
                        self.candies_left
                    ));
                } else {
                    player.point = new_pos;
                    self.finished = true;
                }
            }
            ObjectType::Mob => {
                self.players.remove(&player_id);
                self.dead_players.push(player_id);
                self.map.clear_object(&player.point);
                return;
            }
            ObjectType::Candy => {
                self.map.clear_object(&player.point);
                player.point = new_pos;
                self.candies_left -= 1;
                self.log(format!("{} candies left", self.candies_left));
            }
            _ => {}
        }
        self.map
            .place_object_with_id(player_id, ObjectType::Player(direction), &player.point);
        self.players.insert(player_id, player);
    }

    pub fn player_shoot(&mut self, player_id: u64) {
        let player = self
            .players
            .get(&player_id)
            .unwrap_or_else(|| panic!("Player {} not found", player_id));
        let pos = player.point.step(player.dir);

        let collider_obj = *self.map.get_object(&pos);
        match collider_obj.type_ {
            ObjectType::Empty => {
                let shot_id = rand::random();
                self.map
                    .place_object_with_id(shot_id, ObjectType::Shot(player.dir), &pos);
                self.shots.insert(
                    shot_id,
                    OrientedPoint {
                        point: pos,
                        dir: player.dir,
                    },
                );
            }
            ObjectType::Mob => {
                self.mobs.remove(&collider_obj.id);
                self.map.clear_object(&pos);
            }
            ObjectType::Player(_) => {
                self.log(format!(
                    "{} killed {}",
                    self.player_names[&player_id], self.player_names[&collider_obj.id]
                ));
                self.players.remove(&collider_obj.id);
                self.dead_players.push(collider_obj.id);
                self.map.clear_object(&pos);
            }
            _ => (),
        }
    }
}
