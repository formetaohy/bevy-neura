use crate::host;
use crate::memory::NeuraMemory;
use crate::runtime::NeuraRuntime;
use bevy::ecs::schedule::MainThreadExecutor;
use bevy::prelude::{Message, World};
use bevy::render::renderer::RenderDevice;
use neura::{HostAdvance, MemoryRequest};

#[derive(Message)]
pub struct NeuraRebuilt;

pub(crate) fn borrow_device(world: &mut World) {
    let device = render_device(world);
    let memory = memory_request(world);
    let advance = advance(world);
    world.insert_resource(host::borrow(&device, memory, advance));
}

pub(crate) fn rebuild_runtime(world: &mut World) {
    let Some(device) = world
        .get_resource::<RenderDevice>()
        .map(|device| device.wgpu_device().clone())
    else {
        return;
    };
    let Some(identity) = world
        .get_resource::<NeuraRuntime>()
        .map(NeuraRuntime::identity)
    else {
        return;
    };
    if host::identity(&device) == identity {
        return;
    }
    let memory = memory_request(world);
    let advance = advance(world);
    world.insert_resource(host::borrow(&device, memory, advance));
    world.write_message(NeuraRebuilt);
}

fn render_device(world: &World) -> wgpu::Device {
    world
        .get_resource::<RenderDevice>()
        .unwrap_or_else(|| {
            panic!(
                "Neura borrows the device the App renders on, and the App carries no render device",
            )
        })
        .wgpu_device()
        .clone()
}

fn memory_request(world: &World) -> MemoryRequest {
    world
        .get_resource::<NeuraMemory>()
        .map_or_else(MemoryRequest::default, |memory| memory.0.clone())
}

fn advance(world: &World) -> Option<HostAdvance> {
    world.get_resource::<MainThreadExecutor>().map(|executor| {
        let executor = executor.0.clone();
        HostAdvance::of(move || {
            if let Some(ticker) = executor.ticker() {
                let _ = ticker.try_tick();
            }
        })
    })
}
