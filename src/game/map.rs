use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::mazes::MAZES;

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "lowercase", tag = "direction")]
pub enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
    pub dir: Direction,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Point {
            x,
            y,
            ..Default::default()
        }
    }
    pub fn turn_and_step(&self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Point::new(self.x, self.y - 1),
            Direction::Down => Point::new(self.x, self.y + 1),
            Direction::Left => Point::new(self.x - 1, self.y),
            Direction::Right => Point::new(self.x + 1, self.y),
        }
    }
    pub fn step(&self) -> Self {
        self.turn_and_step(self.dir).set_dir(self.dir)
    }
    fn set_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }
    pub fn update(&mut self, new_pos: Point) {
        self.x = new_pos.x;
        self.y = new_pos.y;
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ObjectType {
    Wall,
    Player(Direction),
    Shot(Direction),
    Exit,
    Mob,
    Candy,
    Empty,
}

impl ObjectType {
    fn from_char(c: char) -> Self {
        match c {
            '█' => ObjectType::Wall,
            '^' => ObjectType::Player(Direction::Up),
            'v' => ObjectType::Player(Direction::Down),
            '<' => ObjectType::Player(Direction::Left),
            '>' => ObjectType::Player(Direction::Right),
            '|' => ObjectType::Shot(Direction::Up),
            '-' => ObjectType::Shot(Direction::Left),
            'X' => ObjectType::Exit,
            '*' => ObjectType::Mob,
            '⏾' => ObjectType::Candy,
            ' ' => ObjectType::Empty,
            _ => panic!("Unknown character in map template: {}", c),
        }
    }
    fn to_char(&self) -> char {
        match self {
            ObjectType::Wall => '█',
            ObjectType::Player(Direction::Up) => '^',
            ObjectType::Player(Direction::Down) => 'v',
            ObjectType::Player(Direction::Left) => '<',
            ObjectType::Player(Direction::Right) => '>',
            ObjectType::Shot(Direction::Up) => '|',
            ObjectType::Shot(Direction::Down) => '|',
            ObjectType::Shot(Direction::Left) => '-',
            ObjectType::Shot(Direction::Right) => '-',
            ObjectType::Exit => 'X',
            ObjectType::Mob => '*',
            ObjectType::Candy => '⏾',
            ObjectType::Empty => ' ',
        }
    }
}

#[derive(Clone, Copy)]
pub struct MapObject {
    pub id: u64,
    pub type_: ObjectType,
}

impl MapObject {
    fn new(type_: ObjectType) -> Self {
        MapObject { type_, id: 0 }
    }
}

#[derive(Default, Clone)]
pub struct Map {
    map: Vec<Vec<MapObject>>,
    width: usize,
    height: usize,
}

impl Map {
    pub fn new(maze_name: &str) -> Self {
        let template = MAZES.get(maze_name).unwrap();

        let mut map = Vec::new();
        let mut width = 0;
        let mut height = 0;
        for line in template.lines() {
            let row: Vec<MapObject> = line
                .chars()
                .map(|ch| MapObject::new(ObjectType::from_char(ch)))
                .collect();
            width = row.len();
            map.push(row);
            height += 1;
        }
        Map { map, width, height }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn random_empty_point(&self) -> Point {
        let mut rng = rand::thread_rng();
        let mut x = rng.gen_range(1..(self.width - 1));
        let mut y = rng.gen_range(1..(self.height - 1));
        while !matches!(self.map[y][x].type_, ObjectType::Empty) {
            x = rng.gen_range(1..(self.width - 1));
            y = rng.gen_range(1..(self.height - 1));
        }
        Point::new(x, y)
    }

    pub fn format(&self) -> String {
        let mut map = String::new();
        for row in &self.map {
            for object in row {
                map.push(object.type_.to_char());
            }
            map.push('\r');
            map.push('\n');
        }
        map
    }

    pub fn place_objects(mut self, objects: Vec<(ObjectType, Point)>) -> Self {
        for (type_, pos) in objects {
            self.map[pos.y][pos.x] = MapObject::new(type_);
        }
        self
    }

    pub fn place_object_with_id(mut self, id: u64, type_: ObjectType, pos: &Point) -> Self {
        self.map[pos.y][pos.x] = MapObject { id, type_ };
        self
    }

    pub fn place_object(mut self, type_: ObjectType, pos: &Point) -> Self {
        self.map[pos.y][pos.x] = MapObject { id: 0, type_ };
        self
    }

    pub fn get_object(&self, pos: &Point) -> &MapObject {
        &self.map[pos.y][pos.x]
    }

    pub fn clear_object(&mut self, pos: &Point) {
        self.map[pos.y][pos.x] = MapObject::new(ObjectType::Empty);
    }

    pub fn swap_objects(&mut self, pos1: &Point, pos2: &Point) {
        let tmp = self.map[pos1.y][pos1.x];
        self.map[pos1.y][pos1.x] = self.map[pos2.y][pos2.x];
        self.map[pos2.y][pos2.x] = tmp;
    }
}
