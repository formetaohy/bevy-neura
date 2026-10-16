#[path = "../common/mod.rs"]
mod common;

mod archive;
mod canvas;
mod dataset;
mod digit;
mod game;
mod ink;
mod learner;
mod lenet;
mod loading;
mod puzzle;
mod random;
mod recognizer;
mod source;
mod train;
mod view;

use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy_neura::NeuraPlugin;
use neura::{MemoryRequest, RuntimeRequest};

const TITLE: &str = "mnist - bevy-neura";
const DEVICE_HEAP: u64 = 1 << 28;
const READBACK: u64 = 1 << 22;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Phase {
    #[default]
    Loading,
    Ready,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        resolution: (760u32, 620u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .build()
                .disable::<PipelinedRenderingPlugin>(),
            NeuraPlugin::of(RuntimeRequest {
                memory: MemoryRequest {
                    heap_bytes: DEVICE_HEAP,
                    readback_bytes: READBACK,
                    readback_slots: 4,
                    ..MemoryRequest::default()
                },
                ..RuntimeRequest::default()
            }),
            common::LoadingPlugin::new(Phase::Loading, TITLE),
        ))
        .init_state::<Phase>()
        .add_systems(Startup, view::setup)
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(Update, loading::drive.run_if(in_state(Phase::Loading)))
        .add_systems(OnEnter(Phase::Ready), view::board)
        .add_systems(
            Update,
            (game::touch, game::read, game::advance, view::paint)
                .chain()
                .run_if(in_state(Phase::Ready)),
        )
        .run();
}
