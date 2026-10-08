use bevy::prelude::*;

pub const SCENERY: f32 = -24.0;
pub const RIDGE: f32 = -21.0;
pub const GROUND: f32 = -12.0;
pub const PIPE: f32 = 1.0;
pub const PUFF: f32 = 1.6;
pub const BIRD: f32 = 2.0;
pub const RING: f32 = 2.5;
pub const VIGNETTE: f32 = 5.0;
pub const DISPLAY: f32 = 6.0;

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb(0.008, 0.014, 0.035)));
    commands.spawn(Camera2d);
}
