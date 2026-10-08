use crate::bird::Bird;
use crate::game::{Game, Stage, Wingbeat};
use crate::random::Random;
use crate::shape;
use crate::view;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

const STEPS: usize = 24;
const PUFF_LIFE: f32 = 0.55;
const DUST_LIFE: f32 = 0.9;
const RING_LIFE: f32 = 1.1;
const INVITE: f32 = 1.2;
const PUFF: f32 = 12.0;
const RING: f32 = 26.0;
const QUAKE: f32 = 6.0;
const SHAKE: f32 = 0.4;
const AIR: Color = Color::srgb(0.74, 0.86, 1.0);
const DUST: Color = Color::srgb(0.52, 0.64, 0.70);
const PULSE: Color = Color::srgb(1.0, 0.86, 0.56);

#[derive(Clone, Copy, PartialEq)]
enum Hue {
    Air,
    Dust,
    Pulse,
}

struct Ramp {
    steps: Vec<Handle<ColorMaterial>>,
}

impl Ramp {
    fn of(materials: &mut Assets<ColorMaterial>, color: Color) -> Self {
        let steps = (0..STEPS)
            .map(|step| {
                let fade = 1.0 - step as f32 / (STEPS - 1) as f32;
                materials.add(shape::blend(color.with_alpha(fade)))
            })
            .collect();
        Self { steps }
    }

    fn at(&self, share: f32) -> Handle<ColorMaterial> {
        let index = (share.clamp(0.0, 1.0) * (STEPS - 1) as f32).round() as usize;
        self.steps[index].clone()
    }
}

#[derive(Resource)]
pub struct Art {
    puff: Handle<Mesh>,
    ring: Handle<Mesh>,
    air: Ramp,
    dust: Ramp,
    pulse: Ramp,
}

impl Art {
    fn ramp(&self, hue: Hue) -> &Ramp {
        match hue {
            Hue::Air => &self.air,
            Hue::Dust => &self.dust,
            Hue::Pulse => &self.pulse,
        }
    }
}

#[derive(Component)]
pub struct Fade {
    age: f32,
    life: f32,
    hue: Hue,
    grow: f32,
    drift: Vec2,
}

#[derive(Resource, Default)]
pub struct Quake {
    left: f32,
    span: f32,
}

impl Quake {
    fn kick(&mut self, span: f32) {
        self.left = span;
        self.span = span;
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puff = meshes.add(shape::disc(PUFF, 6, 32, |offset| {
        Color::WHITE.with_alpha(0.9 * (1.0 - (offset.length() / PUFF).clamp(0.0, 1.0)).powf(1.7))
    }));
    let ring = meshes.add(shape::disc(RING, 6, 48, |offset| {
        let reach = offset.length() / RING;
        Color::WHITE.with_alpha((-((reach - 0.88) / 0.12).powi(2)).exp() * 0.9)
    }));
    commands.insert_resource(Art {
        puff,
        ring,
        air: Ramp::of(&mut materials, AIR),
        dust: Ramp::of(&mut materials, DUST),
        pulse: Ramp::of(&mut materials, PULSE),
    });
    commands.insert_resource(Quake::default());
}

pub fn beat(
    mut commands: Commands,
    art: Res<Art>,
    mut beats: MessageReader<Wingbeat>,
    birds: Query<&Transform, With<Bird>>,
    mut random: Local<Random>,
) {
    for _ in beats.read() {
        for place in &birds {
            for puff in 0..3 {
                let angle = PI * (1.08 + 0.21 * puff as f32);
                let push = Vec2::from_angle(angle);
                spark(
                    &mut commands,
                    &art,
                    &art.puff,
                    Fade {
                        age: 0.0,
                        life: PUFF_LIFE + 0.05 * puff as f32,
                        hue: Hue::Air,
                        grow: 1.1 + 0.5 * puff as f32,
                        drift: push * random.uniform(34.0, 58.0),
                    },
                    place.translation.xy() + push * (9.0 + 3.0 * puff as f32),
                    view::PUFF,
                );
            }
        }
    }
}

pub fn watch(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    bird: Query<&Transform, With<Bird>>,
    mut quake: ResMut<Quake>,
    mut shown: Local<Option<Stage>>,
) {
    let stage = game.stage();
    let previous = shown.replace(stage);
    if previous.is_none() || previous == Some(stage) {
        return;
    }
    match stage {
        Stage::Falling => quake.kick(SHAKE),
        Stage::Over => {
            quake.kick(SHAKE * 0.6);
            for place in &bird {
                burst(&mut commands, &art, place.translation.xy());
            }
        }
        Stage::Ready | Stage::Play => {}
    }
}

pub fn invite(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    time: Res<Time>,
    bird: Query<&Transform, With<Bird>>,
    mut wait: Local<f32>,
) {
    if !matches!(game.stage(), Stage::Ready | Stage::Over) {
        *wait = 0.0;
        return;
    }
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = INVITE;
    for place in &bird {
        spark(
            &mut commands,
            &art,
            &art.ring,
            Fade {
                age: 0.0,
                life: RING_LIFE,
                hue: Hue::Pulse,
                grow: 1.9,
                drift: Vec2::ZERO,
            },
            place.translation.xy(),
            view::RING,
        );
    }
}

pub fn advance(
    time: Res<Time>,
    mut commands: Commands,
    art: Res<Art>,
    mut fades: Query<(
        Entity,
        &mut Fade,
        &mut Transform,
        &mut MeshMaterial2d<ColorMaterial>,
    )>,
) {
    let seconds = time.delta_secs();
    for (entity, mut fade, mut place, mut material) in &mut fades {
        fade.age += seconds;
        let share = fade.age / fade.life;
        if share >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        place.translation.x += fade.drift.x * seconds;
        place.translation.y += fade.drift.y * seconds;
        place.scale = Vec3::splat(1.0 + fade.grow * share);
        material.0 = art.ramp(fade.hue).at(share);
    }
}

pub fn shake(
    time: Res<Time>,
    mut quake: ResMut<Quake>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
) {
    quake.left = (quake.left - time.delta_secs()).max(0.0);
    let share = if quake.span > 0.0 {
        quake.left / quake.span
    } else {
        0.0
    };
    let elapsed = time.elapsed_secs();
    let reach = QUAKE * share * share;
    let offset = Vec2::new((elapsed * 63.0).sin(), (elapsed * 81.0).cos()) * reach;
    for mut place in &mut cameras {
        place.translation = offset.extend(place.translation.z);
    }
}

fn burst(commands: &mut Commands, art: &Art, at: Vec2) {
    for grain in 0..9 {
        let angle = TAU * grain as f32 / 9.0 + 0.35;
        let reach = if grain % 2 == 0 { 16.0 } else { 9.0 };
        spark(
            commands,
            art,
            &art.puff,
            Fade {
                age: 0.0,
                life: DUST_LIFE,
                hue: Hue::Dust,
                grow: 1.2,
                drift: Vec2::from_angle(angle) * reach,
            },
            at + Vec2::new(0.0, 4.0),
            view::PUFF,
        );
    }
}

fn spark(
    commands: &mut Commands,
    art: &Art,
    mesh: &Handle<Mesh>,
    fade: Fade,
    at: Vec2,
    layer: f32,
) {
    let material = art.ramp(fade.hue).at(0.0);
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material),
        Transform::from_xyz(at.x, at.y, layer),
        fade,
    ));
}
