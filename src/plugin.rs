use crate::host;
use crate::memory::NeuraMemory;
use crate::queue;
use bevy::ecs::schedule::MainThreadExecutor;
use bevy::prelude::{App, Plugin, PreStartup, World};
use bevy::render::renderer::RenderDevice;
use neura::{HostAdvance, MemoryRequest};

pub struct NeuraPlugin;

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, borrow_device);
    }

    fn finish(&self, app: &mut App) {
        queue::hold(app);
    }
}

fn borrow_device(world: &mut World) {
    let advance = world.get_resource::<MainThreadExecutor>().map(|executor| {
        let executor = executor.0.clone();
        HostAdvance::of(move || {
            if let Some(ticker) = executor.ticker() {
                let _ = ticker.try_tick();
            }
        })
    });
    let memory = world
        .remove_resource::<NeuraMemory>()
        .map_or_else(MemoryRequest::default, |memory| memory.0);
    let device = world
        .get_resource::<RenderDevice>()
        .unwrap_or_else(|| {
            panic!(
                "Neura borrows the device the App renders on, and the App carries no render device",
            )
        })
        .wgpu_device()
        .clone();
    world.insert_resource(host::borrow(&device, memory, advance));
}
