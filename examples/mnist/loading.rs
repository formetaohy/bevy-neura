use crate::archive;
use crate::dataset::{Archives, Dataset};
use crate::game::Game;
use crate::train::{Progress, Training};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use bevy_neura_example::app::Phase;
use bevy_neura_example::loading::Loading;
use bevy_neura_example::transfer::Transfer;

const PLAN: [(&str, f32); 4] = [
    ("downloading", 0.16),
    ("unpacking", 0.02),
    ("training", 0.79),
    ("saving", 0.03),
];

#[derive(Resource)]
pub(crate) struct Pending {
    cached: bool,
    transfer: Option<Transfer>,
    archives: Option<Archives>,
    training: Option<Training>,
}

pub fn setup(mut commands: Commands) {
    let directory = archive::directory();
    let cached = archive::checkpoint().is_file();
    let transfer = if cached {
        None
    } else {
        Transfer::start(&directory, archive::artifacts())
    };
    commands.insert_resource(Loading::of(&PLAN));
    commands.insert_resource(Pending {
        cached,
        transfer,
        archives: None,
        training: None,
    });
}

pub fn drive(
    runtime: Res<NeuraRuntime>,
    mut commands: Commands,
    mut page: ResMut<Loading>,
    mut pending: ResMut<Pending>,
    mut phase: ResMut<NextState<Phase>>,
) {
    if !page.shown() {
        return;
    }
    match page.index() {
        0 => page.follow(&mut pending.transfer),
        1 => {
            if !pending.cached {
                pending.archives = Some(archive::unpack(&archive::directory()));
            }
            page.report(1.0);
        }
        2 => {
            if pending.training.is_none() {
                pending.training = Some(match pending.archives.take() {
                    Some(archives) => Training::learning(&runtime, Dataset::of(archives)),
                    None => Training::loaded(&runtime, &archive::checkpoint()),
                });
            }
            let training = pending
                .training
                .as_mut()
                .expect("the page of mnist walks the learning it holds");
            match training.advance(&runtime) {
                Progress::Working(part) => page.report(part),
                Progress::Done => page.report(1.0),
            }
        }
        3 => {
            let training = pending
                .training
                .take()
                .expect("the page of mnist keeps the learning it holds");
            training.save(&runtime, &archive::checkpoint());
            commands.insert_resource(Game::of(training.recognizer()));
            commands.remove_resource::<Pending>();
            page.report(1.0);
            phase.set(Phase::Ready);
        }
        steps => panic!("the page of mnist walks {} steps, not {steps}", PLAN.len()),
    }
}
