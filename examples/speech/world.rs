use bevy::prelude::*;

pub const WIDTH: f32 = 720.0;
pub const HEIGHT: f32 = 700.0;
pub const CEILING: f32 = HEIGHT / 2.0;
pub const BAND: f32 = 80.0;
pub const FLOOR: f32 = -HEIGHT / 2.0 + BAND;
pub const SKY: Color = Color::srgb(0.05, 0.07, 0.14);

const GROUND: Color = Color::srgb(0.11, 0.15, 0.24);
const RIM: Color = Color::srgb(0.22, 0.3, 0.44);
const RIM_SIZE: f32 = 6.0;

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(SKY));
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(GROUND, Vec2::new(WIDTH, BAND)),
        Transform::from_xyz(0.0, FLOOR - BAND / 2.0, 0.0),
    ));
    commands.spawn((
        Sprite::from_color(RIM, Vec2::new(WIDTH, RIM_SIZE)),
        Transform::from_xyz(0.0, FLOOR - RIM_SIZE / 2.0, 0.5),
    ));
}
