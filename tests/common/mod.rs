use bevy::app::PluginsState;
use bevy::log::LogPlugin;
use bevy::prelude::{App, DefaultPlugins, PluginGroup, default};
use bevy::render::RenderPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;
use bevy_neura::{NeuraMemory, NeuraPlugin};
use neura::MemoryRequest;
use std::sync::Mutex;

pub fn app(memory: MemoryRequest) -> App {
    static ONE_INSTANCE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let _guard = ONE_INSTANCE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(settings())),
                ..default()
            })
            .build()
            .disable::<WinitPlugin>()
            .disable::<LogPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    );
    app.add_plugins(NeuraPlugin);
    app.insert_resource(NeuraMemory(memory));
    while app.plugins_state() == PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    app
}

const NATIVE: wgpu::Backends = if cfg!(any(target_os = "macos", target_os = "ios")) {
    wgpu::Backends::METAL
} else {
    wgpu::Backends::VULKAN
};

fn settings() -> WgpuSettings {
    WgpuSettings {
        backends: Some(wgpu::Backends::from_env().unwrap_or(NATIVE)),
        ..default()
    }
}
