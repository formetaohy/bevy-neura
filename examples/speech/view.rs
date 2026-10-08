use bevy::prelude::*;

const BACKGROUND: Color = Color::srgb(0.035, 0.045, 0.065);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(BACKGROUND));
    commands.spawn(Camera2d);
}
