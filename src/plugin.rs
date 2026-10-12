use crate::host;
use crate::memory::NeuraMemory;
use bevy::prelude::{App, Plugin, PreStartup, World};
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::RenderDevice;
use neura::MemoryRequest;

pub struct NeuraPlugin;

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, borrow_device);
    }

    fn cleanup(&self, app: &mut App) {
        assert!(
            !app.is_plugin_added::<PipelinedRenderingPlugin>(),
            "the App pipelines its rendering onto a thread of its own, and Neura submits to the queue of the render device from the thread the App drives its world on: disable PipelinedRenderingPlugin",
        );
    }
}

fn borrow_device(world: &mut World) {
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
    world.insert_resource(host::borrow(&device, memory));
}
