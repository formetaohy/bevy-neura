mod bird;
mod column;
mod digit;
mod drift;
mod effect;
mod flap;
mod game;
mod ground;
mod loading;
mod mel;
mod microphone;
mod model;
mod pipe;
mod plume;
mod random;
mod resample;
mod ridge;
mod score;
mod shape;
mod sky;
mod source;
mod speech;
mod tokenizer;
mod utterance;
mod view;
mod voice;
mod world;

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
                    title: "voice flappy bird - say flap - bevy-neura".to_string(),
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
        .add_message::<game::Wingbeat>()
        .add_systems(
            Startup,
            (
                view::setup,
                sky::setup,
                ridge::setup,
                ground::setup,
                column::setup,
                bird::spawn,
                plume::dress,
                effect::setup,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                drift::sweep,
                sky::twinkle,
                digit::paint,
                effect::shake,
                column::dress,
            ),
        )
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(OnExit(Phase::Loading), loading::clear)
        .add_systems(
            Update,
            (loading::drive, loading::paint, loading::breathe)
                .chain()
                .run_if(in_state(Phase::Loading)),
        )
        .add_systems(OnEnter(Phase::Playing), (score::setup, voice::setup))
        .add_systems(
            Update,
            (
                (speech::listen, speech::decode, game::direct, game::advance).chain(),
                (
                    plume::animate,
                    effect::beat,
                    effect::watch,
                    effect::invite,
                    effect::advance,
                    score::paint,
                    voice::paint,
                )
                    .chain(),
            )
                .chain()
                .run_if(in_state(Phase::Playing)),
        )
        .run();
}
