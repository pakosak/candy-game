use chrono::Local;
use std::collections::{HashMap, HashSet};

use crate::game::map::{Direction, Map, ObjectType, Point};

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

#[derive(serde::Serialize)]
pub struct WorldState<'a> {
    pub objects: Vec<(ObjectType, Point)>,
    pub winner: Option<u64>,
    pub dead_players: Vec<u64>,
    pub logs: &'a Vec<String>,
}

#[derive(Default)]
pub struct World {
    map_template: Map,
    players: HashMap<u64, Point>,
    mobs: HashMap<u64, Point>,
    candies: HashMap<u64, Point>,
    shots: HashMap<u64, Point>,

    winner: Option<u64>,
    dead_players: Vec<u64>,
    player_names: HashMap<u64, String>,
    logs: Vec<String>,
}

impl World {
    pub fn new(width: usize, height: usize, mob_cnt: usize, candy_cnt: usize) -> Self {
        let mut map = Map::new(width, height);

        let exit_pos = map.random_empty_point();
        map = map.place_object(ObjectType::Exit, &exit_pos);

        let mut used_points: HashSet<Point> = HashSet::new();

        let candies = (0..candy_cnt)
            .map(|_| loop {
                let candy_pos = map.random_empty_point();
                if used_points.insert(candy_pos) {
                    break (rand::random(), candy_pos);
                }
            })
            .collect();

        let mobs = (0..mob_cnt)
            .map(|_| loop {
                let mob_pos = map.random_empty_point();
                if used_points.insert(mob_pos) {
                    break (rand::random(), mob_pos);
                }
            })
            .collect();

        World {
            map_template: map,
            mobs,
            candies,
            ..Default::default()
        }
    }

    pub fn can_play(&self, player_id: u64) -> bool {
        self.winner.is_none() && !self.dead_players.contains(&player_id)
    }

    pub fn get_map_template(&self) -> Map {
        self.map_template.clone()
    }

    pub fn get_state(&self) -> WorldState {
        WorldState {
            objects: self.object_positions(),
            winner: self.winner,
            dead_players: self.dead_players.clone(),
            logs: &self.logs,
        }
    }

    fn object_positions(&self) -> Vec<(ObjectType, Point)> {
        let mut positions: Vec<(ObjectType, Point)> = Vec::new();
        for player in self.players.values() {
            positions.push((ObjectType::Player(player.dir), *player));
        }
        for mob in self.mobs.values() {
            positions.push((ObjectType::Mob, *mob));
        }
        for candy in self.candies.values() {
            positions.push((ObjectType::Candy, *candy));
        }
        for shot in self.shots.values() {
            positions.push((ObjectType::Shot(shot.dir), *shot));
        }
        positions
    }

    pub fn width(&self) -> usize {
        self.map_template.width()
    }

    pub fn height(&self) -> usize {
        self.map_template.height()
    }

    fn log(&mut self, msg: String) {
        self.logs
            .push(format!("{}: {}", Local::now().format("%H:%M:%S"), msg));
    }

    fn fill_map(&self, mut map: Map) -> Map {
        for (id, player) in &self.players {
            map = map.place_object_with_id(*id, ObjectType::Player(player.dir), player);
        }
        for (id, mob) in &self.mobs {
            map = map.place_object_with_id(*id, ObjectType::Mob, mob);
        }
        for (id, candy) in &self.candies {
            map = map.place_object_with_id(*id, ObjectType::Candy, candy);
        }
        for (id, shot) in &self.shots {
            map = map.place_object_with_id(*id, ObjectType::Shot(shot.dir), shot);
        }
        map
    }

    pub fn move_world(&mut self) {
        let mut map = self.fill_map(self.get_map_template());

        map = self.move_random_mob(map);
        self.move_shots(map);
    }

    pub fn move_random_mob(&mut self, mut map: Map) -> Map {
        let mob_cnt = self.mobs.len();
        if mob_cnt == 0 {
            return map;
        }
        let random_mob_id = *self
            .mobs
            .keys()
            .nth(rand::random::<usize>() % mob_cnt)
            .unwrap();
        let mob_pos = self.mobs.remove(&random_mob_id).unwrap();

        let new_pos = mob_pos.turn_and_step(rand::random());
        let collided_obj = map.get_object(&new_pos);
        match collided_obj.type_ {
            ObjectType::Empty => {
                map.swap_objects(&mob_pos, &new_pos);
                self.mobs.insert(random_mob_id, new_pos);
            }
            ObjectType::Player(_) => {
                self.log(format!(
                    "{} killed by mob",
                    self.player_names[&collided_obj.id]
                ));
                self.players.remove(&collided_obj.id);
                self.dead_players.push(collided_obj.id);
                map.swap_objects(&mob_pos, &new_pos);
                map.clear_object(&mob_pos);
                self.mobs.insert(random_mob_id, new_pos);
            }
            _ => {
                self.mobs.insert(random_mob_id, mob_pos);
            }
        }
        map
    }

    pub fn move_shots(&mut self, mut map: Map) -> Map {
        let mut new_logs: Vec<String> = Vec::new();
        self.shots.retain(|_, shot| {
            let new_pos = shot.step();
            let collider_obj = map.get_object(&new_pos);
            match collider_obj.type_ {
                ObjectType::Empty => {
                    map.swap_objects(shot, &new_pos);
                    shot.update(new_pos);
                    true
                }
                ObjectType::Mob => {
                    new_logs.push("Mob killed by stray shot".to_string());
                    self.mobs.remove(&collider_obj.id);
                    map.clear_object(shot);
                    map.clear_object(&new_pos);
                    false
                }
                ObjectType::Player(_) => {
                    new_logs.push(format!(
                        "{} killed by stray shot",
                        self.player_names[&collider_obj.id]
                    ));
                    self.players.remove(&collider_obj.id);
                    self.dead_players.push(collider_obj.id);
                    map.clear_object(shot);
                    map.clear_object(&new_pos);
                    false
                }
                _ => {
                    map.clear_object(shot);
                    false
                }
            }
        });
        for log in new_logs {
            self.log(log);
        }
        map
    }

    pub fn spawn_player(&mut self, player_name: &str) -> u64 {
        let map = self.fill_map(self.get_map_template());

        let player = map.random_empty_point();
        let player_id = rand::random();
        self.players.insert(player_id, player);
        self.player_names.insert(player_id, player_name.to_string());
        self.log(format!("Player {} entered world", player_name));
        player_id
    }

    pub fn move_player(&mut self, player_id: u64, direction: Direction) {
        let map = self.fill_map(self.get_map_template());

        let mut player = self
            .players
            .remove(&player_id)
            .unwrap_or_else(|| panic!("Player {} not found", player_id));
        player.dir = direction;
        let new_pos = player.step();
        let collider_obj = map.get_object(&new_pos);
        match collider_obj.type_ {
            ObjectType::Empty => {
                player = new_pos;
            }
            ObjectType::Exit => {
                if self.candies.is_empty() {
                    self.log(format!(
                        "You need to collect {} more candies, {}",
                        self.candies.len(),
                        self.player_names[&player_id]
                    ));
                } else {
                    self.log(format!(
                        "Player {} won the game",
                        self.player_names[&player_id]
                    ));
                    player = new_pos; // remove player
                    self.winner = Some(player_id);
                }
            }
            ObjectType::Mob => {
                self.log(format!("{} killed by mob", self.player_names[&player_id]));
                self.players.remove(&player_id);
                self.dead_players.push(player_id);
                return;
            }
            ObjectType::Candy => {
                player = new_pos;
                self.candies.remove(&collider_obj.id);
                self.log(format!("{} candies left", self.candies.len()));
            }
            _ => {}
        }
        self.players.insert(player_id, player);
    }

    pub fn player_shoot(&mut self, player_id: u64) {
        let map = self.fill_map(self.get_map_template());

        let player = self
            .players
            .get(&player_id)
            .unwrap_or_else(|| panic!("Player {} not found", player_id));
        let pos = player.step();

        let collider_obj = map.get_object(&pos);
        match collider_obj.type_ {
            ObjectType::Empty => {
                let shot_id = rand::random();
                self.shots.insert(shot_id, pos);
            }
            ObjectType::Mob => {
                self.mobs.remove(&collider_obj.id);
            }
            ObjectType::Player(_) => {
                self.log(format!(
                    "{} killed {}",
                    self.player_names[&player_id], self.player_names[&collider_obj.id]
                ));
                self.players.remove(&collider_obj.id);
                self.dead_players.push(collider_obj.id);
            }
            _ => (),
        }
    }

    pub fn erase_player(&mut self, player_id: u64) {
        self.log(format!(
            "Player {} left the game",
            self.player_names[&player_id]
        ));
        self.players.remove(&player_id);
        self.player_names.remove(&player_id);
        self.dead_players.retain(|player| *player != player_id);
    }
}
