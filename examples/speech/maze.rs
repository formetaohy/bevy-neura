use bevy::prelude::*;

pub const TILE: f32 = 32.0;

pub const LAYOUT: [&str; 17] = [
    "#####################",
    "#.........#.........#",
    "#.###.###.#.###.###.#",
    "#o###.###.#.###.###o#",
    "#...................#",
    "#.....#.##.##.#.....#",
    "#.....#.##-##.#.....#",
    "#.###.#.#   #.#.###.#",
    "#.....#.#   #.#.....#",
    "#.....#.#####.#.....#",
    "#.###.#.#####.#.###.#",
    "#.....#...#...#.....#",
    "#.###.###.#.###.###.#",
    "#o..#.....P.....#..o#",
    "###.#.#.#####.#.#.###",
    "#.....#...#...#.....#",
    "#####################",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Door,
    Floor,
    House,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pellet {
    Dot,
    Power,
}

pub struct Maze;

impl Maze {
    pub const COLS: i32 = LAYOUT[0].len() as i32;
    pub const ROWS: i32 = LAYOUT.len() as i32;

    pub fn tile(at: IVec2) -> Tile {
        match Self::letter(at) {
            b'#' => Tile::Wall,
            b'-' => Tile::Door,
            b' ' => Tile::House,
            b'.' | b'o' | b'P' => Tile::Floor,
            letter => panic!("the layout answers {letter} where a maze holds #, -, space, . or o"),
        }
    }

    pub fn pellet(at: IVec2) -> Option<Pellet> {
        match Self::letter(at) {
            b'.' => Some(Pellet::Dot),
            b'o' => Some(Pellet::Power),
            _ => None,
        }
    }

    pub fn passable(from: IVec2, to: IVec2, ghost: bool) -> bool {
        match Self::tile(to) {
            Tile::Wall => false,
            Tile::Door => ghost && Self::tile(from) == Tile::House,
            Tile::Floor | Tile::House => true,
        }
    }

    pub fn point(at: Vec2) -> Vec2 {
        Vec2::new(
            (at.x - (Self::COLS as f32 - 1.0) / 2.0) * TILE,
            ((Self::ROWS as f32 - 1.0) / 2.0 - at.y) * TILE,
        )
    }

    pub fn center(at: IVec2) -> Vec2 {
        Self::point(at.as_vec2())
    }

    pub fn tiles() -> impl Iterator<Item = (IVec2, Tile)> {
        (0..Self::ROWS)
            .flat_map(|row| (0..Self::COLS).map(move |column| IVec2::new(column, row)))
            .map(|at| (at, Self::tile(at)))
    }

    pub fn start() -> IVec2 {
        Self::tiles()
            .find(|(at, _)| Self::letter(*at) == b'P')
            .map(|(at, _)| at)
            .expect("the layout holds the tile a pac opens on")
    }

    fn letter(at: IVec2) -> u8 {
        if at.x < 0 || at.y < 0 || at.x >= Self::COLS || at.y >= Self::ROWS {
            return b'#';
        }
        LAYOUT[at.y as usize].as_bytes()[at.x as usize]
    }
}
