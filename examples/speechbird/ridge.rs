use crate::drift::Drift;
use crate::shape;
use crate::view;
use crate::world;
use bevy::prelude::*;
use std::f32::consts::TAU;

const PERIOD: f32 = world::WIDTH * 1.4;
const SAMPLES: usize = 96;
const SPAN: f32 = PERIOD / 2.0;
const LAYER: f32 = view::RIDGE;
const BASE: f32 = world::FLOOR - 40.0;
const FAR_SPEED: f32 = 5.0;
const NEAR_SPEED: f32 = 11.0;
const FAR: Color = Color::srgb(0.075, 0.115, 0.20);
const NEAR: Color = Color::srgb(0.035, 0.06, 0.115);
const CREST: Color = Color::srgb(0.62, 0.78, 0.92);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let look = materials.add(ColorMaterial::from(Color::WHITE));
    let far = line(far_line);
    commands.spawn((
        Mesh2d(
            meshes.add(shape::grid(&[far.clone(), floor(&far)], |row, _, at| {
                if row == 0 {
                    shape::mix(FAR, CREST, 0.18 * rise(at.y))
                } else {
                    FAR
                }
            })),
        ),
        MeshMaterial2d(look.clone()),
        Transform::from_xyz(0.0, 0.0, LAYER),
        Drift::new(FAR_SPEED, SPAN),
    ));
    commands.spawn((
        Mesh2d(meshes.add(shape::grid(
            &[raise(&far, 1.7), raise(&far, -1.7)],
            |_, _, _| CREST.with_alpha(0.45),
        ))),
        MeshMaterial2d(materials.add(shape::blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, LAYER + 0.1),
        Drift::new(FAR_SPEED, SPAN),
    ));
    let near = line(near_line);
    commands.spawn((
        Mesh2d(meshes.add(shape::grid(&[near.clone(), floor(&near)], |_, _, _| NEAR))),
        MeshMaterial2d(look),
        Transform::from_xyz(0.0, 0.0, LAYER + 0.2),
        Drift::new(NEAR_SPEED, SPAN),
    ));
}

fn line(profile: impl Fn(f32) -> f32) -> Vec<Vec2> {
    (0..=SAMPLES)
        .map(|sample| PERIOD * (sample as f32 / SAMPLES as f32 - 0.5))
        .map(|at| Vec2::new(at, profile(at)))
        .collect()
}

fn floor(of: &[Vec2]) -> Vec<Vec2> {
    of.iter().map(|at| Vec2::new(at.x, BASE)).collect()
}

fn raise(of: &[Vec2], by: f32) -> Vec<Vec2> {
    of.iter().map(|at| Vec2::new(at.x, at.y + by)).collect()
}

fn rise(at: f32) -> f32 {
    ((at - BASE) / 150.0).clamp(0.0, 1.0)
}

fn wave(at: f32, harmonic: f32, phase: f32) -> f32 {
    (TAU * harmonic * at / PERIOD + phase).sin()
}

fn far_line(at: f32) -> f32 {
    world::FLOOR
        + 112.0
        + 30.0 * wave(at, 1.0, 0.0)
        + 15.0 * wave(at, 2.0, 1.3)
        + 7.0 * wave(at, 3.0, 2.6)
}

fn near_line(at: f32) -> f32 {
    world::FLOOR + 58.0 + 20.0 * wave(at, 2.0, 2.1) + 9.0 * wave(at, 5.0, 0.7)
}
