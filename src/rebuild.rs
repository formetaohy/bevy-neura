use crate::host;
use crate::plugin::NeuraRequest;
use crate::runtime::NeuraRuntime;
use bevy::ecs::schedule::MainThreadExecutor;
use bevy::prelude::{Message, World};
use bevy::render::renderer::RenderDevice;
use neura::HostAdvance;

#[derive(Message)]
pub struct NeuraRebuilt;

pub(crate) fn borrow_runtime(world: &mut World) {
    let device = render_device(world);
    if world
        .get_resource::<NeuraRuntime>()
        .is_some_and(|runtime| runtime.answers_to(&device))
    {
        return;
    }
    let replaced = world.contains_resource::<NeuraRuntime>();
    let request = world.resource::<NeuraRequest>().requested();
    let advance = advance(world);
    world.insert_resource(host::borrow(&device, request, advance));
    if replaced {
        world.write_message(NeuraRebuilt);
    }
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
