use crate::Phase;
use crate::common::Loading;
use crate::microphone::Microphone;
use crate::model::{DecoderModel, EncoderModel};
use crate::source::{self, Report};
use crate::speech::{Checkpoint, Speech};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const PLAN: [(&str, f32); 5] = [
    ("downloading the checkpoint", 0.6),
    ("reading the checkpoint", 0.1),
    ("building the encoder", 0.1),
    ("building the decoder", 0.1),
    ("opening the microphone", 0.1),
];

#[derive(Resource)]
pub(crate) struct Pending {
    transfer: Option<source::Transfer>,
    checkpoint: Option<Checkpoint>,
    encoder: Option<EncoderModel>,
    decoder: Option<DecoderModel>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Loading::of(&PLAN));
    commands.insert_resource(Pending {
        transfer: source::Transfer::start(&source::directory()),
        checkpoint: None,
        encoder: None,
        decoder: None,
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
        0 => match pending.transfer.as_ref().map(source::Transfer::report) {
            None => page.report(1.0),
            Some(Report::Done) => {
                pending.transfer = None;
                page.report(1.0);
            }
            Some(Report::Failed(error)) => panic!("the checkpoint does not download: {error}"),
            Some(Report::Measuring) => page.report(0.0),
            Some(Report::Fetching { done, total }) if total > 0 => {
                page.report(done as f32 / total as f32);
            }
            Some(Report::Fetching { .. }) => page.report(0.0),
        },
        1 => {
            pending.checkpoint = Some(Checkpoint::read(&source::directory()));
            page.report(1.0);
        }
        2 => {
            let checkpoint = pending
                .checkpoint
                .as_ref()
                .expect("the page of speech holds the checkpoint it reads");
            pending.encoder = Some(EncoderModel::load(
                &runtime,
                &checkpoint.dims,
                checkpoint.weights(),
            ));
            page.report(1.0);
        }
        3 => {
            let checkpoint = pending
                .checkpoint
                .as_ref()
                .expect("the page of speech holds the checkpoint it reads");
            pending.decoder = Some(DecoderModel::load(
                &runtime,
                &checkpoint.dims,
                checkpoint.weights(),
            ));
            page.report(1.0);
        }
        4 => {
            let speech = Speech::open(
                Microphone::open(),
                pending
                    .checkpoint
                    .take()
                    .expect("the page of speech holds the checkpoint it opens"),
                pending
                    .encoder
                    .take()
                    .expect("the page of speech holds the encoder it opens"),
                pending
                    .decoder
                    .take()
                    .expect("the page of speech holds the decoder it opens"),
            );
            commands.insert_resource(speech);
            commands.remove_resource::<Pending>();
            page.report(1.0);
            phase.set(Phase::Ready);
        }
        steps => panic!("the page of speech walks {} steps, not {steps}", PLAN.len()),
    }
}
