mod console;
mod loading;
mod mel;
mod microphone;
mod model;
mod resample;
mod source;
mod speech;
mod tokenizer;
mod transcript;
mod utterance;
mod view;

use bevy::prelude::*;
use bevy_neura::NeuraPlugin;
use neura::{MemoryRequest, RuntimeRequest};

const DEVICE_HEAP: u64 = 1 << 30;
const READBACK: u64 = 1 << 23;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Phase {
    #[default]
    Loading,
    Ready,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "speech - hold the button and talk - bevy-neura".to_string(),
                    resolution: (660u32, 720u32).into(),
                    ..default()
                }),
                ..default()
            }),
            NeuraPlugin::new(RuntimeRequest {
                memory: MemoryRequest {
                    heap_bytes: DEVICE_HEAP,
                    readback_bytes: READBACK,
                    readback_slots: 4,
                },
                ..RuntimeRequest::default()
            }),
        ))
        .init_state::<Phase>()
        .add_message::<utterance::Utterance>()
        .add_systems(Startup, view::setup)
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(OnExit(Phase::Loading), loading::clear)
        .add_systems(
            Update,
            (loading::drive, loading::paint)
                .chain()
                .run_if(in_state(Phase::Loading)),
        )
        .add_systems(OnEnter(Phase::Ready), (transcript::setup, console::setup))
        .add_systems(
            Update,
            (
                speech::capture,
                speech::decode,
                transcript::write,
                transcript::paint,
                console::paint,
            )
                .chain()
                .run_if(in_state(Phase::Ready)),
        )
        .run();
}
