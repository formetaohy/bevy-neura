mod board;
mod game;
mod learner;
mod loading;
mod net;
mod replay;
mod training;
mod view;

use bevy::prelude::*;
use bevy_neura::NeuraPlugin;
use bevy_neura_example::app::{self, Phase};
use bevy_neura_example::loading::LoadingPlugin;
use bevy_neura_example::stage;

const TITLE: &str = "tic tac toe";
const DEVICE_HEAP: u64 = 1 << 28;
const READBACK: u64 = 1 << 20;
const WINDOW: (u32, u32) = (480, 500);

fn main() {
    App::new()
        .add_plugins((
            app::windowed(TITLE, WINDOW),
            NeuraPlugin::of(app::request(DEVICE_HEAP, READBACK)),
            LoadingPlugin::new(Phase::Loading, TITLE),
        ))
        .init_state::<Phase>()
        .add_systems(Startup, stage::setup)
        .add_systems(OnEnter(Phase::Loading), loading::setup)
        .add_systems(Update, loading::drive.run_if(in_state(Phase::Loading)))
        .add_systems(OnEnter(Phase::Ready), view::setup)
        .add_systems(
            Update,
            (game::advance, game::click, view::paint)
                .chain()
                .run_if(in_state(Phase::Ready)),
        )
        .run();
}
