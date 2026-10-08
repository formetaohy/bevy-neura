use crate::Phase;
use crate::microphone::Microphone;
use crate::model::{DecoderModel, EncoderModel};
use crate::source::{self, Report};
use crate::speech::{Checkpoint, Speech};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const DOWNLOAD: f32 = 0.6;
const WIDTH: f32 = 360.0;
const TRACK_HEIGHT: f32 = 8.0;
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
const TITLE: Color = Color::srgb(0.94, 0.97, 1.0);
const DIM: Color = Color::srgb(0.55, 0.62, 0.76);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Download,
    Checkpoint,
    Encoder,
    Decoder,
    Microphone,
}

impl Step {
    const ALL: [Self; 5] = [
        Self::Download,
        Self::Checkpoint,
        Self::Encoder,
        Self::Decoder,
        Self::Microphone,
    ];

    fn share(self) -> f32 {
        match self {
            Self::Download => DOWNLOAD,
            _ => (1.0 - DOWNLOAD) / (Self::ALL.len() - 1) as f32,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Download => "downloading the checkpoint",
            Self::Checkpoint => "reading the checkpoint",
            Self::Encoder => "building the encoder",
            Self::Decoder => "building the decoder",
            Self::Microphone => "opening the microphone",
        }
    }
}

#[derive(Resource)]
pub struct Loading {
    transfer: Option<source::Transfer>,
    step: Step,
    checkpoint: Option<Checkpoint>,
    encoder: Option<EncoderModel>,
    decoder: Option<DecoderModel>,
}

impl Loading {
    fn progress(&self) -> f32 {
        let done = Step::ALL
            .iter()
            .take_while(|step| **step != self.step)
            .map(|step| step.share())
            .sum::<f32>();
        let partial = match (&self.transfer, self.step) {
            (Some(transfer), Step::Download) => match transfer.report() {
                Report::Fetching { done, total } if total > 0 => {
                    DOWNLOAD * done as f32 / total as f32
                }
                _ => 0.0,
            },
            (None, Step::Download) => DOWNLOAD,
            _ => 0.0,
        };
        (done + partial).min(1.0)
    }
}

#[derive(Component)]
pub(crate) struct Screen;

#[derive(Component)]
pub(crate) struct Fill;

pub(crate) enum Part {
    Label,
    Percent,
}

#[derive(Component)]
pub(crate) struct Mark(Part);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Loading {
        transfer: source::Transfer::start(&source::directory()),
        step: Step::Download,
        checkpoint: None,
        encoder: None,
        decoder: None,
    });
    commands.spawn((
        Screen,
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new(source::MODEL),
                TextFont::from_font_size(34.0),
                TextColor(TITLE),
            ),
            (
                Mark(Part::Label),
                Text::new(Step::Download.label()),
                TextFont::from_font_size(18.0),
                TextColor(DIM),
            ),
            (
                Node {
                    width: Val::Px(WIDTH),
                    height: Val::Px(TRACK_HEIGHT),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(TRACK),
                children![(
                    Fill,
                    Node {
                        width: Val::Px(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(FILL),
                )],
            ),
            (
                Mark(Part::Percent),
                Text::new("0%"),
                TextFont::from_font_size(18.0),
                TextColor(DIM),
            ),
        ],
    ));
}

pub fn clear(mut commands: Commands, screens: Query<Entity, With<Screen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Loading>();
}

pub fn drive(
    runtime: Res<NeuraRuntime>,
    mut commands: Commands,
    mut loading: ResMut<Loading>,
    mut phase: ResMut<NextState<Phase>>,
    mut frames: Local<u32>,
) {
    *frames += 1;
    if *frames < 3 {
        return;
    }
    if let Some(transfer) = &loading.transfer {
        match transfer.report() {
            Report::Done => loading.transfer = None,
            Report::Failed(error) => panic!("the checkpoint does not download: {error}"),
            Report::Measuring | Report::Fetching { .. } => {}
        }
        return;
    }
    match loading.step {
        Step::Download => loading.step = Step::Checkpoint,
        Step::Checkpoint => {
            loading.checkpoint = Some(Checkpoint::read(&source::directory()));
            loading.step = Step::Encoder;
        }
        Step::Encoder => {
            let checkpoint = loading
                .checkpoint
                .as_ref()
                .expect("a loading checkpoint holds its numbers");
            loading.encoder = Some(EncoderModel::load(
                &runtime,
                &checkpoint.dims,
                checkpoint.weights(),
            ));
            loading.step = Step::Decoder;
        }
        Step::Decoder => {
            let checkpoint = loading
                .checkpoint
                .as_ref()
                .expect("a loading checkpoint holds its numbers");
            loading.decoder = Some(DecoderModel::load(
                &runtime,
                &checkpoint.dims,
                checkpoint.weights(),
            ));
            loading.step = Step::Microphone;
        }
        Step::Microphone => {
            let speech = Speech::open(
                Microphone::open(),
                loading
                    .checkpoint
                    .take()
                    .expect("a loading checkpoint holds its numbers"),
                loading
                    .encoder
                    .take()
                    .expect("a loading encoder holds its graph"),
                loading
                    .decoder
                    .take()
                    .expect("a loading decoder holds its graph"),
            );
            commands.insert_resource(speech);
            phase.set(Phase::Ready);
        }
    }
}

pub fn paint(
    loading: Res<Loading>,
    mut fills: Query<&mut Node, With<Fill>>,
    mut marks: Query<(&Mark, &mut Text)>,
) {
    let progress = loading.progress();
    for mut node in &mut fills {
        node.width = Val::Px(WIDTH * progress);
    }
    let percent = format!("{}%", (progress * 100.0).round() as u32);
    for (mark, mut text) in &mut marks {
        match mark.0 {
            Part::Label => {
                let words = loading.step.label();
                if text.0 != words {
                    text.0 = words.to_string();
                }
            }
            Part::Percent => {
                if text.0 != percent {
                    text.0 = percent.clone();
                }
            }
        }
    }
}
