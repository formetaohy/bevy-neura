use crate::theme;
use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(theme::BACKGROUND));
    commands.spawn(Camera2d);
}
