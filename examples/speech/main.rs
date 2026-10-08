mod direction;
mod game;
mod ghost;
mod hud;
mod loading;
mod maze;
mod mel;
mod microphone;
mod model;
mod resample;
mod source;
mod speech;
mod tokenizer;
mod utterance;
mod walker;

use bevy::prelude::*;
use bevy_neura::NeuraPlugin;
use neura::{MemoryRequest, RuntimeRequest};

const DEVICE_HEAP: u64 = 1 << 30;
const READBACK: u64 = 1 << 23;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Phase {
    #[default]
    Loading,
    Playing,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "voice pac-man - bevy-neura".to_string(),
                    resolution: (720u32, 700u32).into(),
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
        .init_resource::<game::Game>()
        .add_message::<utterance::Utterance>()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.03, 0.06)))
        .add_systems(Startup, (game::setup, hud::setup))
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(OnExit(Phase::Loading), loading::clear)
        .add_systems(
            Update,
            (loading::drive, loading::paint)
                .chain()
                .run_if(in_state(Phase::Loading)),
        )
        .add_systems(
            Update,
            (
                speech::listen,
                speech::decode,
                game::direct,
                game::advance,
                game::face,
                hud::paint,
            )
                .chain()
                .run_if(in_state(Phase::Playing)),
        )
        .run();
}
