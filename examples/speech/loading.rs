use crate::Phase;
use crate::microphone::Microphone;
use crate::model::{DecoderModel, EncoderModel};
use crate::source::{self, Report};
use crate::speech::{Checkpoint, Speech};
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const DOWNLOAD: f32 = 0.6;
const BACKDROP: Color = Color::srgb(0.02, 0.02, 0.04);
const TRACK: Color = Color::srgb(0.12, 0.14, 0.22);
const TEXT: Color = Color::srgb(0.82, 0.86, 0.95);
const DIM: Color = Color::srgb(0.5, 0.56, 0.7);
const READY: Color = Color::srgb(0.98, 0.85, 0.2);

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
        let partial = match self.step {
            Step::Download => match &self.transfer {
                Some(transfer) => match transfer.report() {
                    Report::Fetching { done, total } if total > 0 => {
                        DOWNLOAD * done as f32 / total as f32
                    }
                    _ => 0.0,
                },
                None => DOWNLOAD,
            },
            _ => 0.0,
        };
        (done + partial).min(1.0)
    }

    fn caption(&self) -> String {
        let mut lines = vec![
            "voice flappy bird".to_string(),
            "whisper reads the microphone in the frame loop".to_string(),
            String::new(),
        ];
        for step in Step::ALL {
            let marker = if (step as usize) < self.step as usize {
                "[x]"
            } else if step == self.step {
                "[>]"
            } else {
                "[ ]"
            };
            lines.push(format!("{marker} {}", self.detail(step)));
        }
        lines.join("\n")
    }

    fn detail(&self, step: Step) -> String {
        match step {
            Step::Download => match &self.transfer {
                None => format!("{} is already on disk", source::MODEL),
                Some(transfer) => match transfer.report() {
                    Report::Measuring => format!("{}: asking for its size", source::MODEL),
                    Report::Fetching { done, total } => {
                        format!("{}: {} MB of {} MB", source::MODEL, mega(done), mega(total))
                    }
                    Report::Done => format!("{} is here", source::MODEL),
                    Report::Failed(error) => error,
                },
            },
            Step::Checkpoint => "the checkpoint, vocabulary and mel banks".to_string(),
            Step::Encoder => "the encoder behind the microphone".to_string(),
            Step::Decoder => "the decoder that names the words".to_string(),
            Step::Microphone => "the microphone that feeds the bird".to_string(),
        }
    }
}

#[derive(Component)]
pub struct Screen;

#[derive(Component)]
pub struct Steps;

#[derive(Component)]
pub struct Fill;

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
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(BACKDROP),
    ));
    spawn_text(&mut commands, "voice flappy bird", 34.0, 170.0, READY);
    spawn_text(
        &mut commands,
        "whisper reads the microphone in the frame loop",
        18.0,
        232.0,
        DIM,
    );
    commands.spawn((
        Screen,
        Steps,
        Text::new(""),
        font(18.0),
        TextColor(TEXT),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(150.0),
            top: Val::Px(290.0),
            ..default()
        },
    ));
    let fill = commands
        .spawn((
            Fill,
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(READY),
        ))
        .id();
    commands
        .spawn((
            Screen,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(150.0),
                top: Val::Px(500.0),
                width: Val::Px(420.0),
                height: Val::Px(16.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(TRACK),
            BorderColor::all(TRACK),
        ))
        .add_child(fill);
}

fn spawn_text(commands: &mut Commands, text: &str, size: f32, top: f32, color: Color) {
    commands.spawn((
        Screen,
        Text::new(text),
        font(size),
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(150.0),
            top: Val::Px(top),
            ..default()
        },
    ));
}

fn font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
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
        let report = transfer.report();
        match report {
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
            phase.set(Phase::Playing);
        }
    }
}

pub fn paint(
    loading: Res<Loading>,
    mut steps: Query<&mut Text, With<Steps>>,
    mut fills: Query<&mut Node, With<Fill>>,
) {
    for mut text in &mut steps {
        text.0 = loading.caption();
    }
    for mut node in &mut fills {
        node.width = Val::Percent(loading.progress() * 100.0);
    }
}

fn mega(bytes: u64) -> u64 {
    bytes / (1 << 20)
}
