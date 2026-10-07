mod command;
mod game;
mod mel;
mod microphone;
mod model;
mod resample;
mod source;
mod speech;
mod tokenizer;

use bevy::prelude::*;
use bevy_neura::{NeuraPlugin, NeuraRuntime};
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
                    title: "voice commander - bevy-neura".to_string(),
                    resolution: (960u32, 660u32).into(),
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
        .add_message::<speech::Transcription>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
        .add_systems(Startup, game::setup)
        .add_systems(Update, load.run_if(in_state(Phase::Loading)))
        .add_systems(
            Update,
            (
                speech::listen,
                speech::decode,
                game::act,
                game::steer,
                game::spawn,
                game::march,
                game::strike,
                game::paint,
            )
                .run_if(in_state(Phase::Playing)),
        )
        .run();
}

fn load(
    runtime: Res<NeuraRuntime>,
    mut commands: Commands,
    mut phase: ResMut<NextState<Phase>>,
    mut frames: Local<u32>,
) {
    *frames += 1;
    if *frames < 2 {
        return;
    }
    commands.insert_resource(speech::Speech::load(&runtime));
    phase.set(Phase::Playing);
}
