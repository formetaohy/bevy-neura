use crate::world;
use bevy::prelude::*;

pub const START_X: f32 = -230.0;
pub const START_Y: f32 = -30.0;
pub const RADIUS: f32 = 16.0;

const GRAVITY: f32 = 200.0;
const LIFT: f32 = 180.0;
const FALL: f32 = 220.0;
const BOB_RATE: f32 = 5.0;
const BOB_SIZE: f32 = 9.0;

#[derive(Component)]
pub struct Bird {
    velocity: f32,
}

impl Bird {
    fn new() -> Self {
        Self { velocity: 0.0 }
    }

    pub fn home(&mut self, at: &mut Transform) {
        self.velocity = 0.0;
        at.translation = Vec3::new(START_X, START_Y, at.translation.z);
        at.rotation = Quat::IDENTITY;
    }

    pub fn flap(&mut self) {
        self.velocity = LIFT;
    }

    pub fn hover(&mut self, seconds: f32, at: &mut Transform) {
        self.velocity = 0.0;
        at.translation.x = START_X;
        at.translation.y = START_Y + (seconds * BOB_RATE).sin() * BOB_SIZE;
    }

    pub fn fall(&mut self, dt: f32, at: &mut Transform) -> bool {
        self.velocity = (self.velocity - GRAVITY * dt).max(-FALL);
        at.translation.y += self.velocity * dt;
        if at.translation.y > world::CEILING - RADIUS {
            at.translation.y = world::CEILING - RADIUS;
            self.velocity = 0.0;
        }
        at.translation.y - RADIUS <= world::FLOOR
    }

    pub fn tilt(&self, at: &mut Transform) {
        at.rotation = Quat::from_rotation_z((self.velocity / FALL).atan());
    }
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Bird::new(),
        Visibility::Inherited,
        Transform::from_xyz(START_X, START_Y, 0.0),
    ));
}
