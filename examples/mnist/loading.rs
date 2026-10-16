use crate::Phase;
use crate::archive;
use crate::common::Loading;
use crate::dataset::{Archives, Dataset};
use crate::game::Game;
use crate::source::{Report, Transfer};
use crate::train::{Progress, Training};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const PLAN: [(&str, f32); 4] = [
    ("downloading the digits", 0.16),
    ("reading the digits", 0.02),
    ("learning the digits", 0.79),
    ("keeping the learning", 0.03),
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
    let transfer = (!cached && !archive::held(&directory)).then(|| Transfer::start(&directory));
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
        0 => match pending.transfer.as_ref().map(Transfer::report) {
            None => page.report(1.0),
            Some(Report::Done) => {
                pending.transfer = None;
                page.report(1.0);
            }
            Some(Report::Failed(error)) => panic!("the digits do not download: {error}"),
            Some(Report::Measuring) => page.report(0.0),
            Some(Report::Fetching { done, total }) if total > 0 => {
                page.report(done as f32 / total as f32);
            }
            Some(Report::Fetching { .. }) => page.report(0.0),
        },
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
