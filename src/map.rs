use rand::prelude::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct OrientedPoint {
    pub point: Point,
    pub dir: Direction,
}

#[derive(Copy, Clone, Default, Debug)]
pub enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum ObjectType {
    Wall,
    Player(Direction),
    Shot(Direction),
    Exit,
    Mob,
    Candy,
    Empty,
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

#[derive(Default)]
pub struct Map {
    map: Vec<Vec<MapObject>>,
    width: usize,
    height: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Map {
            map: vec![vec![MapObject::new(ObjectType::Empty); width]; height],
            width,
            height,
        }
        .build_walls()
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
        Point { x, y }
    }

    pub fn format(&self) -> String {
        let mut map = String::new();
        for row in &self.map {
            for object in row {
                match object.type_ {
                    ObjectType::Wall => map.push('#'),
                    ObjectType::Player(dir) => match dir {
                        Direction::Up => map.push('^'),
                        Direction::Down => map.push('v'),
                        Direction::Left => map.push('<'),
                        Direction::Right => map.push('>'),
                    },
                    ObjectType::Shot(dir) => match dir {
                        Direction::Up => map.push('|'),
                        Direction::Down => map.push('|'),
                        Direction::Left => map.push('-'),
                        Direction::Right => map.push('-'),
                    },
                    ObjectType::Exit => map.push('X'),
                    ObjectType::Mob => map.push('*'),
                    ObjectType::Candy => map.push('C'),
                    ObjectType::Empty => map.push(' '),
                }
            }
            map.push('\r');
            map.push('\n');
        }
        map
    }

    pub fn place_object_with_id(&mut self, id: u64, type_: ObjectType, pos: &Point) {
        self.map[pos.y][pos.x] = MapObject { id, type_ };
    }

    pub fn place_object(&mut self, type_: ObjectType, pos: &Point) {
        self.map[pos.y][pos.x] = MapObject { id: 0, type_ };
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

    fn build_walls(mut self) -> Self {
        for row in &mut self.map {
            row[0] = MapObject::new(ObjectType::Wall);
            row[self.width - 1] = MapObject::new(ObjectType::Wall);
        }
        for i in 0..self.width {
            self.map[0][i] = MapObject::new(ObjectType::Wall);
            self.map[self.height - 1][i] = MapObject::new(ObjectType::Wall);
        }

        self
    }
}
