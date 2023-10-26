use std::collections::HashMap;

use crate::map::{Direction, Map, ObjectType, OrientedPoint, Point};

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

#[derive(Default)]
pub struct World {
    map: Map,
    player: OrientedPoint,
    exit: Point,
    mobs: HashMap<u64, Point>,
    shots: HashMap<u64, OrientedPoint>,

    win: Option<bool>,
    candies_left: usize,
    log: Vec<String>,
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

        w.player = OrientedPoint {
            point: w.map.random_empty_point(),
            dir: Direction::Right,
        };
        w.map
            .place_object(ObjectType::Player(Direction::Right), &w.player.point);
        w.log(format!("Player placed at {:?}", w.player));

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

    pub fn win_status(&self) -> Option<bool> {
        self.win
    }

    pub fn map_string(&self) -> String {
        self.map.format()
    }

    pub fn get_logs(&self) -> &Vec<String> {
        self.log.as_ref()
    }

    pub fn width(&self) -> usize {
        self.map.width()
    }

    pub fn height(&self) -> usize {
        self.map.height()
    }

    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }

    pub fn move_random_mob(&mut self) {
        let mob_cnt = self.mobs.len();
        if mob_cnt == 0 {
            return;
        }
        if let Some((_, mob_pos)) = self.mobs.iter_mut().nth(rand::random::<usize>() % mob_cnt) {
            let new_pos = mob_pos.step(rand::random());
            let obj = self.map.get_object(&new_pos);
            match obj.type_ {
                ObjectType::Empty => {
                    self.map.swap_objects(mob_pos, &new_pos);
                    mob_pos.update(new_pos);
                }
                ObjectType::Player(_) => {
                    self.map.clear_object(mob_pos);
                    self.win = Some(false);
                }
                _ => (),
            }
        }
    }

    pub fn move_shots(&mut self) {
        self.shots.retain(|_, shot| {
            let new_pos = shot.point.step(shot.dir);
            let obj = self.map.get_object(&new_pos);
            match obj.type_ {
                ObjectType::Empty => {
                    self.map.swap_objects(&shot.point, &new_pos);
                    shot.point.update(new_pos);
                    true
                }
                ObjectType::Mob => {
                    // self.log(format!("Mob killed at {:?}", new_pos));
                    self.mobs.remove(&obj.id);
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
    }

    pub fn move_player(&mut self, direction: Direction) {
        let new_pos = self.player.point.step(direction);
        let obj = self.map.get_object(&new_pos);
        let new_player = OrientedPoint {
            point: new_pos,
            dir: direction,
        };
        match obj.type_ {
            ObjectType::Empty => {
                self.map.clear_object(&self.player.point);
                self.player = new_player;
                self.map
                    .place_object(ObjectType::Player(direction), &self.player.point);
            }
            ObjectType::Exit => {
                if self.candies_left != 0 {
                    self.log(format!(
                        "You need to collect {} more candies",
                        self.candies_left
                    ));
                    return;
                }
                self.player = new_player;
                self.map
                    .place_object(ObjectType::Player(direction), &self.player.point);
                self.win = Some(true);
            }
            ObjectType::Mob => {
                self.map.clear_object(&self.player.point);
                self.win = Some(false);
            }
            ObjectType::Candy => {
                self.map.clear_object(&self.player.point);
                self.player = new_player;
                self.map
                    .place_object(ObjectType::Player(direction), &self.player.point);
                self.candies_left -= 1;
                self.log(format!("{} candies left", self.candies_left));
            }
            _ => (),
        }
    }

    pub fn player_shoot(&mut self) {
        let pos = self.player.point.step(self.player.dir);
        if matches!(self.map.get_object(&pos).type_, ObjectType::Empty) {
            let shot_id = rand::random();
            self.map
                .place_object_with_id(shot_id, ObjectType::Shot(self.player.dir), &pos);
            self.shots.insert(
                shot_id,
                OrientedPoint {
                    point: pos,
                    dir: self.player.dir,
                },
            );
        }
    }
}