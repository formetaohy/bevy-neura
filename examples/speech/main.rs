mod command;
mod game;
mod hud;
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

const READBACK: u64 = 1 << 23;

fn heap() -> u64 {
    let megabytes = std::env::var("SPEECH_HEAP")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1024);
    megabytes * 1024 * 1024
}

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
                    title: "voice commander — bevy-neura".to_string(),
                    resolution: (960u32, 660u32).into(),
                    ..default()
                }),
                ..default()
            }),
            NeuraPlugin::new(RuntimeRequest {
                memory: MemoryRequest {
                    heap_bytes: heap(),
                    readback_bytes: READBACK,
                    readback_slots: 4,
                },
                ..RuntimeRequest::default()
            }),
        ))
        .init_state::<Phase>()
        .init_resource::<game::Game>()
        .init_resource::<game::Spawner>()
        .init_resource::<speech::PushToTalk>()
        .add_message::<speech::Transcription>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
        .add_systems(Startup, (game::setup, hud::setup))
        .add_systems(Update, load.run_if(in_state(Phase::Loading)))
        .add_systems(
            Update,
            (
                speech::listen,
                speech::decode,
                game::apply,
                game::steer,
                game::spawn,
                game::march,
                game::strike,
                game::fade,
                game::thaw,
                hud::update,
                hud::talk,
                hud::restart,
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
    let speech = speech::Speech::load(&runtime);
    commands.insert_resource(speech);
    phase.set(Phase::Playing);
}
