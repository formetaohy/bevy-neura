use crate::shape;
use crate::speech::{Speech, State};
use crate::view;
use crate::world;
use bevy::prelude::*;

const LAYER: f32 = view::DISPLAY;
const AT: Vec2 = Vec2::new(0.0, world::FLOOR - 40.0);
const CORE: f32 = 13.0;
const RING: f32 = 20.0;
const LOUD: f32 = 0.085;
const IDLE: Color = Color::srgb(0.30, 0.36, 0.52);
const RECORD: Color = Color::srgb(0.35, 0.90, 0.98);
const READ: Color = Color::srgb(1.0, 0.72, 0.32);
const WRITE: Color = Color::srgb(0.74, 0.62, 1.0);

#[derive(Component)]
pub struct Core;

#[derive(Component)]
pub struct Ring;

type Rings<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut MeshMaterial2d<ColorMaterial>,
    ),
    (With<Ring>, Without<Core>),
>;

#[derive(Resource)]
pub struct Hue([Handle<ColorMaterial>; 4]);

impl Hue {
    fn of(&self, state: State) -> Handle<ColorMaterial> {
        let slot = match state {
            State::Idle => 0,
            State::Recording => 1,
            State::Reading => 2,
            State::Transcribing => 3,
        };
        self.0[slot].clone()
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let core = meshes.add(shape::disc(CORE, 8, 48, |offset| {
        Color::WHITE
            .with_alpha(0.16 + 0.84 * (1.0 - (offset.length() / CORE).clamp(0.0, 1.0)).powf(1.3))
    }));
    let ring = meshes.add(shape::disc(RING, 5, 48, |offset| {
        let reach = offset.length() / RING;
        Color::WHITE.with_alpha((-((reach - 0.9) / 0.09).powi(2)).exp() * 0.85)
    }));
    commands.spawn((
        Ring,
        Mesh2d(ring),
        MeshMaterial2d(materials.add(shape::blend(Color::WHITE))),
        Transform::from_xyz(AT.x, AT.y, LAYER),
    ));
    commands.spawn((
        Core,
        Mesh2d(core),
        MeshMaterial2d(materials.add(shape::blend(Color::WHITE))),
        Transform::from_xyz(AT.x, AT.y, LAYER + 0.1),
    ));
    commands.insert_resource(Hue([
        materials.add(shape::blend(IDLE)),
        materials.add(shape::blend(RECORD)),
        materials.add(shape::blend(READ)),
        materials.add(shape::blend(WRITE)),
    ]));
}

pub fn paint(
    time: Res<Time>,
    speech: Res<Speech>,
    hue: Res<Hue>,
    mut cores: Query<(&mut Transform, &mut MeshMaterial2d<ColorMaterial>), With<Core>>,
    mut rings: Rings,
) {
    let elapsed = time.elapsed_secs();
    let state = speech.state();
    let level = (speech.level() / LOUD).sqrt().clamp(0.0, 1.0);
    let thinking = if matches!(state, State::Reading | State::Transcribing) {
        0.16 * (0.5 + 0.5 * (elapsed * 6.0).sin())
    } else {
        0.0
    };
    for (mut place, mut material) in &mut cores {
        material.0 = hue.of(state);
        place.scale = Vec3::splat(0.70 + 0.70 * level + thinking);
    }
    for (mut place, mut material) in &mut rings {
        material.0 = hue.of(state);
        place.scale = Vec3::splat(0.86 + 0.34 * level + 0.04 * (elapsed * 1.7).sin());
    }
}
