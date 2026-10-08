use crate::drift::Drift;
use crate::pipe;
use crate::shape;
use crate::view;
use crate::world;
use bevy::prelude::*;

const LAYER: f32 = view::GROUND;
const SPAN: f32 = world::WIDTH * 0.75;
const BASE: f32 = -world::HEIGHT / 2.0 - 30.0;
const HATCHES: usize = 17;
const HATCH_STEP: f32 = 48.0;
const HATCH_SPAN: f32 = 384.0;
const HATCH_SIZE: Vec2 = Vec2::new(3.0, 17.0);
const TOP: Color = Color::srgb(0.10, 0.155, 0.235);
const MID: Color = Color::srgb(0.055, 0.085, 0.15);
const DEEP: Color = Color::srgb(0.02, 0.035, 0.07);
const RIM: Color = Color::srgb(0.42, 0.88, 0.78);
const HATCH: Color = Color::srgb(0.45, 0.85, 0.80);
const RIM_SIZE: f32 = 3.0;
const GLOW: f32 = 26.0;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let body = [world::FLOOR, world::FLOOR - 34.0, BASE]
        .iter()
        .map(|at| {
            [-SPAN, SPAN]
                .iter()
                .map(|x| Vec2::new(*x, *at))
                .collect::<Vec<Vec2>>()
        })
        .collect::<Vec<Vec<Vec2>>>();
    commands.spawn((
        Mesh2d(meshes.add(shape::grid(&body, |row, _, _| match row {
            0 => TOP,
            1 => MID,
            _ => DEEP,
        }))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, LAYER),
    ));
    let rim = [
        world::FLOOR + 0.5,
        world::FLOOR - RIM_SIZE,
        world::FLOOR - GLOW,
    ]
    .iter()
    .map(|at| {
        [-SPAN, SPAN]
            .iter()
            .map(|x| Vec2::new(*x, *at))
            .collect::<Vec<Vec2>>()
    })
    .collect::<Vec<Vec<Vec2>>>();
    commands.spawn((
        Mesh2d(meshes.add(shape::grid(&rim, |row, _, _| match row {
            0 => RIM.with_alpha(0.95),
            1 => RIM.with_alpha(0.75),
            _ => RIM.with_alpha(0.0),
        }))),
        MeshMaterial2d(materials.add(shape::blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, LAYER + 0.1),
    ));
    for hatch in 0..HATCHES {
        commands.spawn((
            Sprite::from_color(HATCH.with_alpha(0.14), HATCH_SIZE),
            Transform::from_xyz(
                -HATCH_SPAN + hatch as f32 * HATCH_STEP,
                world::FLOOR - 22.0,
                LAYER + 0.2,
            ),
            Drift::new(pipe::SPEED, HATCH_SPAN),
        ));
    }
}
