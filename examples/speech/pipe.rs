use crate::world;
use bevy::prelude::*;

pub const WIDTH: f32 = 78.0;
pub const GAP: f32 = 300.0;
pub const SPEED: f32 = 120.0;
pub const PERIOD: f32 = 2.5;
pub const DRIFT: f32 = 60.0;
pub const LOW: f32 = world::FLOOR + GAP / 2.0 + MARGIN;
pub const HIGH: f32 = world::CEILING - GAP / 2.0 - MARGIN;
pub const SPAWN: f32 = world::WIDTH / 2.0 + WIDTH / 2.0;

const MARGIN: f32 = 60.0;
const PIPE: Color = Color::srgb(0.28, 0.6, 0.42);

#[derive(Component)]
#[require(Visibility)]
pub struct Pipe {
    pub gap: f32,
    scored: bool,
}

impl Pipe {
    pub fn at(gap: f32) -> Self {
        Self { gap, scored: false }
    }

    pub fn passed(&mut self, at: f32, bird: f32) -> bool {
        if self.scored {
            return false;
        }
        self.scored = at + WIDTH / 2.0 < bird;
        self.scored
    }

    pub fn touches(&self, at: Vec2, bird: Vec2, radius: f32) -> bool {
        let near = |low: f32, high: f32| {
            let point = Vec2::new(
                bird.x.clamp(at.x - WIDTH / 2.0, at.x + WIDTH / 2.0),
                bird.y.clamp(low, high),
            );
            point.distance_squared(bird) < radius * radius
        };
        near(world::FLOOR, self.gap - GAP / 2.0) || near(self.gap + GAP / 2.0, world::CEILING)
    }
}

pub fn spawn(commands: &mut Commands, gap: f32) {
    let lower = gap - GAP / 2.0 - world::FLOOR;
    let upper = world::CEILING - gap - GAP / 2.0;
    commands
        .spawn((Pipe::at(gap), Transform::from_xyz(SPAWN, 0.0, 1.0)))
        .with_children(|pipes| {
            pipes.spawn((
                Sprite::from_color(PIPE, Vec2::new(WIDTH, lower)),
                Transform::from_xyz(0.0, world::FLOOR + lower / 2.0, 0.0),
            ));
            pipes.spawn((
                Sprite::from_color(PIPE, Vec2::new(WIDTH, upper)),
                Transform::from_xyz(0.0, world::CEILING - upper / 2.0, 0.0),
            ));
        });
}
