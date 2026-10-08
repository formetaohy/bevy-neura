mod body;
mod contact;
mod craft;
mod display;
mod env;
mod exhaust;
mod ground;
mod hud;
mod learner;
mod net;
mod particles;
mod policy;
mod random;
mod rollout;
mod shape;
mod sky;
mod terrain;
mod train;
mod world;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy_neura::NeuraPlugin;
use neura::{MemoryRequest, RuntimeRequest};

const DEVICE_HEAP: u64 = 1 << 28;
const READBACK: u64 = 1 << 22;
const WINDOW: (u32, u32) = (1000, 680);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "lunar lander - PPO on bevy-neura".to_string(),
                    resolution: WINDOW.into(),
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
        .insert_resource(ClearColor(Color::srgb(0.01, 0.012, 0.03)))
        .add_systems(
            Startup,
            (
                camera,
                train::build,
                display::build.after(train::build),
                (
                    sky::setup,
                    ground::setup,
                    craft::setup,
                    exhaust::setup,
                    hud::setup,
                )
                    .chain()
                    .after(display::build),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                keys,
                train::advance,
                display::advance,
                ground::refresh,
                sky::twinkle,
                craft::paint,
                exhaust::paint,
                ground::pulse,
                hud::paint,
            )
                .chain(),
        )
        .run();
}

fn camera(mut commands: Commands) {
    let projection = OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: terrain::WIDTH + 2.0,
            min_height: terrain::HEIGHT + 1.4,
        },
        ..OrthographicProjection::default_2d()
    };
    commands.spawn((
        Camera2d,
        Projection::Orthographic(projection),
        Transform::from_xyz(terrain::WIDTH / 2.0, terrain::HEIGHT / 2.0, 0.0),
    ));
}

fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut training: ResMut<train::Training>,
    mut display: ResMut<display::Display>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        training.steps = if training.steps == train::SLOW {
            train::TURBO
        } else {
            train::SLOW
        };
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        training.steps = if training.steps == 0 { train::SLOW } else { 0 };
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        display.replay();
    }
}
