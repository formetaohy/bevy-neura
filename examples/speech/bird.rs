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
const BODY: Color = Color::srgb(0.98, 0.82, 0.25);
const BEAK: Color = Color::srgb(0.95, 0.55, 0.2);
const EYE: Color = Color::srgb(0.06, 0.08, 0.12);

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

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            Bird::new(),
            Mesh2d(meshes.add(Circle::new(RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from(BODY))),
            Transform::from_xyz(START_X, START_Y, 2.0),
        ))
        .with_children(|bird| {
            bird.spawn((
                Mesh2d(meshes.add(Triangle2d::new(
                    Vec2::new(4.0, 7.0),
                    Vec2::new(4.0, -7.0),
                    Vec2::new(19.0, 0.0),
                ))),
                MeshMaterial2d(materials.add(ColorMaterial::from(BEAK))),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            bird.spawn((
                Mesh2d(meshes.add(Circle::new(4.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from(EYE))),
                Transform::from_xyz(4.0, 6.0, 0.2),
            ));
        });
}
