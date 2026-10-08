use crate::Phase;
use crate::digit::{self, Glyphs, Style};
use crate::microphone::Microphone;
use crate::model::{DecoderModel, EncoderModel};
use crate::shape;
use crate::source::{self, Report};
use crate::speech::{Checkpoint, Speech};
use crate::view;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const DOWNLOAD: f32 = 0.6;
const BAR: Vec2 = Vec2::new(420.0, 14.0);
const BAR_Y: f32 = 6.0;
const DIGITS_Y: f32 = 88.0;
const PIPS_Y: f32 = -50.0;
const PIPS: usize = 5;
const PIP: Vec2 = Vec2::new(18.0, 6.0);
const PIP_STEP: f32 = 26.0;
const GLOW: f32 = 190.0;
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
const FILL_HOT: Color = Color::srgb(1.0, 0.82, 0.44);
const HEAD: Color = Color::srgb(1.0, 0.98, 0.92);
const PIP_ON: Color = Color::srgb(0.52, 0.92, 0.86);
const PIP_OFF: Color = Color::srgb(0.14, 0.18, 0.27);
const HALO: Color = Color::srgb(0.30, 0.72, 0.86);

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
}

#[derive(Component)]
pub struct Screen;

#[derive(Component)]
pub enum Part {
    Fill,
    Head,
    Pip(usize),
}

#[derive(Component)]
pub struct Percent;

#[derive(Component)]
pub struct Glow;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Loading {
        transfer: source::Transfer::start(&source::directory()),
        step: Step::Download,
        checkpoint: None,
        encoder: None,
        decoder: None,
    });
    let halo = meshes.add(shape::disc(GLOW, 6, 48, |offset| {
        let reach = offset.length() / GLOW;
        Color::WHITE.with_alpha((-((reach - 0.45) / 0.36).powi(2)).exp())
    }));
    let screen = commands
        .spawn((
            Screen,
            Visibility::Inherited,
            Transform::from_xyz(0.0, 0.0, view::DISPLAY),
        ))
        .id();
    commands.entity(screen).with_children(|screen| {
        screen.spawn((
            Glow,
            Mesh2d(halo),
            MeshMaterial2d(materials.add(shape::blend(HALO.with_alpha(0.10)))),
            Transform::from_xyz(0.0, 16.0, 0.0),
        ));
        screen.spawn((
            Sprite::from_color(TRACK, BAR),
            Transform::from_xyz(0.0, BAR_Y, 0.05),
        ));
        screen.spawn((
            Part::Fill,
            Sprite::from_color(FILL, Vec2::new(1.0, BAR.y)),
            Transform::from_xyz(-BAR.x / 2.0, BAR_Y, 0.08),
        ));
        screen.spawn((
            Part::Head,
            Sprite::from_color(HEAD, Vec2::new(4.0, BAR.y + 8.0)),
            Transform::from_xyz(-BAR.x / 2.0, BAR_Y, 0.1),
        ));
        for pip in 0..PIPS {
            screen.spawn((
                Part::Pip(pip),
                Sprite::from_color(PIP_OFF, PIP),
                Transform::from_xyz(
                    -PIP_STEP * (PIPS - 1) as f32 / 2.0 + pip as f32 * PIP_STEP,
                    PIPS_Y,
                    0.05,
                ),
            ));
        }
    });
    let percent = digit::spawn(
        &mut commands,
        Style {
            at: Vec2::new(0.0, DIGITS_Y),
            dot: 5.5,
            gap: 2.4,
            slot: 9.0,
            slots: 3,
            lit: FILL,
            ghost: PIP_OFF,
            layer: 0.05,
        },
    );
    commands.entity(percent).insert(Percent);
    commands.entity(screen).add_child(percent);
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
    mut parts: Query<(&Part, &mut Sprite, &mut Transform)>,
    mut digits: Query<&mut Glyphs, With<Percent>>,
) {
    let progress = loading.progress();
    let done = loading.step as usize;
    let filled = BAR.x * progress;
    for (part, mut sprite, mut place) in &mut parts {
        match part {
            Part::Fill => {
                sprite.custom_size = Some(Vec2::new(filled.max(1.0), BAR.y));
                place.translation.x = -BAR.x / 2.0 + filled / 2.0;
                sprite.color = shape::mix(FILL, FILL_HOT, progress);
            }
            Part::Head => {
                place.translation.x = -BAR.x / 2.0 + filled;
            }
            Part::Pip(index) => {
                sprite.color = if *index <= done { PIP_ON } else { PIP_OFF };
            }
        }
    }
    for mut glyphs in &mut digits {
        glyphs.show((progress * 100.0).round() as u32);
    }
}

pub fn breathe(time: Res<Time>, mut glows: Query<&mut Transform, With<Glow>>) {
    let pulse = 1.0 + 0.05 * (time.elapsed_secs() * 1.6).sin();
    for mut place in &mut glows {
        place.scale = Vec3::splat(pulse);
    }
}
