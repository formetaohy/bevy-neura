use crate::direction::Direction;
use bevy::prelude::*;

pub const TURNS: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
    Direction::Right,
];

pub fn step_of(direction: Direction) -> IVec2 {
    match direction {
        Direction::Left => IVec2::new(-1, 0),
        Direction::Right => IVec2::new(1, 0),
        Direction::Up => IVec2::new(0, -1),
        Direction::Down => IVec2::new(0, 1),
    }
}

pub fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
    }
}

#[derive(Clone, Copy)]
pub struct Walker {
    pub tile: IVec2,
    heading: Option<Direction>,
    progress: f32,
}

impl Walker {
    pub fn at(tile: IVec2) -> Self {
        Self {
            tile,
            heading: None,
            progress: 0.0,
        }
    }

    pub fn position(&self) -> Vec2 {
        match self.heading {
            Some(heading) => self.tile.as_vec2() + step_of(heading).as_vec2() * self.progress,
            None => self.tile.as_vec2(),
        }
    }

    pub fn moving(&self) -> bool {
        self.heading.is_some()
    }

    pub fn travel(
        &mut self,
        distance: f32,
        wanted: Option<Direction>,
        mut choose: impl FnMut(IVec2, Option<Direction>) -> Option<Direction>,
    ) {
        if let (Some(want), Some(heading)) = (wanted, self.heading)
            && want == opposite(heading)
        {
            self.tile += step_of(heading);
            self.progress = 1.0 - self.progress;
            self.heading = Some(want);
        }
        let mut left = distance;
        loop {
            if self.heading.is_none() {
                self.heading = choose(self.tile, None);
            }
            let Some(heading) = self.heading else {
                return;
            };
            let remaining = 1.0 - self.progress;
            if left < remaining {
                self.progress += left;
                return;
            }
            left -= remaining;
            self.tile += step_of(heading);
            self.progress = 0.0;
            self.heading = choose(self.tile, Some(heading));
        }
    }
}
