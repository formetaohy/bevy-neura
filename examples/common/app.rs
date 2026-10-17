use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::{WindowPlugin, WindowResolution};
use neura::{MemoryRequest, RuntimeRequest};

const READBACK_SLOTS: usize = 4;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    #[default]
    Loading,
    Ready,
}

pub fn windowed(title: &str, resolution: (u32, u32)) -> PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: title.to_string(),
                resolution: WindowResolution::from(resolution),
                ..default()
            }),
            ..default()
        })
        .build()
        .disable::<PipelinedRenderingPlugin>()
}

pub fn request(heap_bytes: u64, readback_bytes: u64) -> RuntimeRequest {
    RuntimeRequest {
        memory: MemoryRequest {
            heap_bytes,
            readback_bytes,
            readback_slots: READBACK_SLOTS,
            ..MemoryRequest::default()
        },
        ..RuntimeRequest::default()
    }
}
