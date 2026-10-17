use crate::microphone::Microphone;
use crate::model::{DecoderModel, EncoderModel};
use crate::source;
use crate::speech::{Checkpoint, Speech};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use bevy_neura_example::app::Phase;
use bevy_neura_example::loading::Loading;
use bevy_neura_example::transfer::Transfer;

const PLAN: [(&str, f32); 5] = [
    ("downloading the checkpoint", 0.6),
    ("reading the checkpoint", 0.1),
    ("building the encoder", 0.1),
    ("building the decoder", 0.1),
    ("opening the microphone", 0.1),
];

#[derive(Resource)]
pub(crate) struct Pending {
    transfer: Option<Transfer>,
    checkpoint: Option<Checkpoint>,
    encoder: Option<EncoderModel>,
    decoder: Option<DecoderModel>,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Loading::of(&PLAN));
    commands.insert_resource(Pending {
        transfer: Transfer::start(&source::directory(), source::artifacts()),
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
        0 => page.follow(&mut pending.transfer),
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
