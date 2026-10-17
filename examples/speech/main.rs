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

use bevy::prelude::*;
use bevy_neura::NeuraPlugin;
use bevy_neura_example::app::{self, Phase};
use bevy_neura_example::loading::LoadingPlugin;
use bevy_neura_example::stage;

const DEVICE_HEAP: u64 = 1 << 30;
const READBACK: u64 = 1 << 23;
const WINDOW: (u32, u32) = (660, 720);

fn main() {
    App::new()
        .add_plugins((
            app::windowed(source::MODEL, WINDOW),
            NeuraPlugin::of(app::request(DEVICE_HEAP, READBACK)),
            LoadingPlugin::new(Phase::Loading, source::MODEL),
        ))
        .init_state::<Phase>()
        .add_message::<utterance::Utterance>()
        .add_systems(Startup, stage::setup)
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(Update, loading::drive.run_if(in_state(Phase::Loading)))
        .add_systems(OnEnter(Phase::Ready), (transcript::setup, console::setup))
        .add_systems(
            Update,
            (
                speech::capture,
                speech::decode,
                transcript::write,
                console::paint,
            )
                .chain()
                .run_if(in_state(Phase::Ready)),
        )
        .run();
}
