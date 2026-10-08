use crate::direction::Direction;
use crate::maze::{Maze, Tile};
use crate::walker::{TURNS, Walker, opposite, step_of};
use bevy::prelude::*;

pub const EXIT: IVec2 = IVec2::new(10, 5);

const TINTS: [Color; 4] = [
    Color::srgb(0.95, 0.35, 0.35),
    Color::srgb(0.95, 0.6, 0.85),
    Color::srgb(0.4, 0.85, 0.9),
    Color::srgb(0.95, 0.7, 0.35),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Scatter,
    Chase,
}

#[derive(Component)]
pub struct Ghost {
    pub walk: Walker,
    pub wait: f32,
    pub tint: Color,
    start: IVec2,
    home: IVec2,
    corner: IVec2,
    release: f32,
}

impl Ghost {
    pub fn roster() -> [Self; 4] {
        [
            Self::new(
                IVec2::new(10, 5),
                IVec2::new(10, 7),
                IVec2::new(19, 1),
                0.0,
                TINTS[0],
            ),
            Self::new(
                IVec2::new(9, 7),
                IVec2::new(9, 7),
                IVec2::new(1, 1),
                2.0,
                TINTS[1],
            ),
            Self::new(
                IVec2::new(11, 7),
                IVec2::new(11, 7),
                IVec2::new(19, 15),
                4.0,
                TINTS[2],
            ),
            Self::new(
                IVec2::new(10, 8),
                IVec2::new(10, 8),
                IVec2::new(1, 15),
                6.0,
                TINTS[3],
            ),
        ]
    }

    fn new(start: IVec2, home: IVec2, corner: IVec2, release: f32, tint: Color) -> Self {
        Self {
            walk: Walker::at(start),
            wait: release,
            tint,
            start,
            home,
            corner,
            release,
        }
    }

    pub fn reset(&mut self) {
        self.walk = Walker::at(self.start);
        self.wait = self.release;
    }

    pub fn send_home(&mut self, wait: f32) {
        self.walk = Walker::at(self.home);
        self.wait = wait;
    }

    pub fn captive(&self) -> bool {
        Maze::tile(self.walk.tile) == Tile::House
    }

    pub fn travel(&mut self, distance: f32, pac: IVec2, mode: Mode, frightened: bool) {
        let captive = self.captive();
        let target = if captive {
            EXIT
        } else {
            match mode {
                Mode::Scatter => self.corner,
                Mode::Chase => pac,
            }
        };
        let fleeing = frightened && !captive;
        self.walk.travel(distance, None, |tile, heading| {
            let open = |direction: Direction| Maze::passable(tile, tile + step_of(direction), true);
            let ahead = TURNS
                .into_iter()
                .filter(|direction| open(*direction) && Some(*direction) != heading.map(opposite))
                .collect::<Vec<Direction>>();
            let steps = if ahead.is_empty() {
                TURNS.into_iter().filter(|step| open(*step)).collect()
            } else {
                ahead
            };
            let mut best = None;
            let mut nearest = i32::MAX;
            for step in steps {
                let gap = key(tile, step, target, fleeing);
                if gap < nearest {
                    nearest = gap;
                    best = Some(step);
                }
            }
            best
        });
    }
}

fn key(tile: IVec2, direction: Direction, target: IVec2, fleeing: bool) -> i32 {
    let gap = (tile + step_of(direction) - target).length_squared();
    if fleeing { -gap } else { gap }
}
