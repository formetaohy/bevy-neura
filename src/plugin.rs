use crate::queue;
use crate::rebuild::{NeuraRebuilt, borrow_device, rebuild_runtime};
use bevy::prelude::{App, First, Plugin, PreStartup};

pub struct NeuraPlugin;

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NeuraRebuilt>();
        app.add_systems(PreStartup, borrow_device);
        app.add_systems(First, rebuild_runtime);
    }

    fn finish(&self, app: &mut App) {
        queue::hold(app);
    }
}
