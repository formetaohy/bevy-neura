use crate::drift::Drift;
use crate::random::Random;
use crate::shape;
use crate::view;
use crate::world;
use bevy::prelude::*;
use std::f32::consts::TAU;

const SPAN: f32 = world::WIDTH * 0.75;
const BOTTOM: f32 = -world::HEIGHT / 2.0 - 24.0;
const TOP: f32 = world::CEILING + 24.0;
const ROWS: u32 = 72;
const HAZE: f32 = 74.0;
const HAZE_WIDTH: f32 = 116.0;
const STARS: usize = 190;
const STAR_SPEED: f32 = 1.6;
const CLOUDS: usize = 7;
const CLOUD_SPEED: f32 = 7.0;
const CLOUD_RADIUS: f32 = 46.0;
const MOON: Vec2 = Vec2::new(236.0, 232.0);
const MOON_RADIUS: f32 = 33.0;
const MOON_LIGHT: Vec2 = Vec2::new(-0.6, 0.8);
const MOON_HALO: f32 = MOON_RADIUS * 4.4;
const VIGNETTE: f32 = 620.0;
const NIGHT: Color = Color::srgb(0.012, 0.02, 0.055);
const HORIZON: Color = Color::srgb(0.075, 0.135, 0.26);
const GLOW: Color = Color::srgb(0.32, 0.34, 0.45);
const MOON_FACE: Color = Color::srgb(0.88, 0.90, 0.82);
const MOON_SHADE: Color = Color::srgb(0.28, 0.31, 0.42);
const MOON_HUE: Color = Color::srgb(0.70, 0.78, 1.0);
const CLOUD: Color = Color::srgb(0.34, 0.44, 0.72);
const STAR_TINTS: [Color; 3] = [
    Color::srgb(0.86, 0.92, 1.0),
    Color::srgb(1.0, 0.95, 0.84),
    Color::srgb(0.86, 0.98, 0.94),
];

#[derive(Component)]
pub struct Star {
    tint: Color,
    phase: f32,
    speed: f32,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(backdrop())),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, view::SCENERY),
    ));
    let mut random = Random::seeded(0x51d3_4a21);
    for _ in 0..STARS {
        let tint = STAR_TINTS[random.below(STAR_TINTS.len())];
        commands.spawn((
            Sprite::from_color(
                tint.with_alpha(random.uniform(0.30, 1.0)),
                Vec2::splat(random.uniform(1.0, 2.6)),
            ),
            Transform::from_xyz(
                random.uniform(-SPAN, SPAN),
                random.uniform(world::FLOOR + 26.0, TOP),
                view::SCENERY + 0.4,
            ),
            Drift::new(STAR_SPEED, SPAN),
            Star {
                tint,
                phase: random.uniform(0.0, TAU),
                speed: random.uniform(0.6, 2.6),
            },
        ));
    }
    commands.spawn((
        Mesh2d(meshes.add(shape::disc(MOON_HALO, 6, 48, |offset| {
            Color::WHITE
                .with_alpha(0.34 * (1.0 - (offset.length() / MOON_HALO).clamp(0.0, 1.0)).powf(3.0))
        }))),
        MeshMaterial2d(materials.add(shape::blend(MOON_HUE.with_alpha(0.55)))),
        Transform::from_xyz(MOON.x, MOON.y, view::SCENERY + 0.5),
    ));
    commands.spawn((
        Mesh2d(meshes.add(shape::disc(MOON_RADIUS, 8, 56, |offset| {
            shape::ball(offset, MOON_RADIUS, MOON_LIGHT, MOON_SHADE, MOON_FACE)
        }))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_xyz(MOON.x, MOON.y, view::SCENERY + 0.6),
    ));
    let crater = materials.add(shape::blend(MOON_SHADE.with_alpha(0.38)));
    for (at, radius) in [
        (Vec2::new(-9.0, 7.0), 6.5),
        (Vec2::new(7.5, -8.0), 8.5),
        (Vec2::new(-3.5, -14.0), 4.2),
        (Vec2::new(12.0, 9.0), 3.4),
    ] {
        commands.spawn((
            Mesh2d(meshes.add(shape::disc(radius, 4, 26, move |offset| {
                Color::WHITE
                    .with_alpha(0.80 * (1.0 - (offset.length() / radius).clamp(0.0, 1.0)).powf(0.8))
            }))),
            MeshMaterial2d(crater.clone()),
            Transform::from_xyz(MOON.x + at.x, MOON.y + at.y, view::SCENERY + 0.7),
        ));
    }
    let cloud = meshes.add(shape::disc(CLOUD_RADIUS, 6, 40, |offset| {
        Color::WHITE.with_alpha((1.0 - (offset.length() / CLOUD_RADIUS).clamp(0.0, 1.0)).powf(2.0))
    }));
    let cloud_look = materials.add(shape::blend(CLOUD.with_alpha(0.14)));
    for _ in 0..CLOUDS {
        commands.spawn((
            Mesh2d(cloud.clone()),
            MeshMaterial2d(cloud_look.clone()),
            Transform::from_xyz(
                random.uniform(-SPAN, SPAN),
                random.uniform(world::FLOOR + 150.0, TOP),
                view::SCENERY + 0.8,
            )
            .with_scale(Vec3::new(
                random.uniform(2.2, 4.6),
                random.uniform(0.45, 0.85),
                1.0,
            )),
            Drift::new(CLOUD_SPEED, SPAN + 260.0),
        ));
    }
    commands.spawn((
        Mesh2d(meshes.add(shape::disc(VIGNETTE, 6, 64, |offset| {
            let reach = (offset.length() / VIGNETTE).clamp(0.0, 1.0);
            Color::BLACK.with_alpha(0.48 * ((reach - 0.42) / 0.58).clamp(0.0, 1.0).powf(1.8))
        }))),
        MeshMaterial2d(materials.add(shape::blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, view::VIGNETTE),
    ));
}

pub fn twinkle(time: Res<Time>, mut stars: Query<(&Star, &mut Sprite)>) {
    let seconds = time.elapsed_secs();
    for (star, mut sprite) in &mut stars {
        let wave = 0.5 + 0.5 * (seconds * star.speed + star.phase).sin();
        sprite.color = star.tint.with_alpha(0.30 + 0.70 * wave);
    }
}

fn backdrop() -> Mesh {
    let row = (TOP - BOTTOM) / ROWS as f32;
    let points = [-SPAN, SPAN]
        .iter()
        .map(|x| {
            (0..=ROWS)
                .map(|step| Vec2::new(*x, BOTTOM + step as f32 * row))
                .collect::<Vec<Vec2>>()
        })
        .collect::<Vec<Vec<Vec2>>>();
    shape::grid(&points, |_, _, at| sky(at.y))
}

fn sky(at: f32) -> Color {
    let rise = ((at - BOTTOM) / (TOP - BOTTOM)).clamp(0.0, 1.0);
    let base = shape::mix(NIGHT, HORIZON, rise.powf(1.8));
    let haze = (-((at - world::FLOOR - HAZE) / HAZE_WIDTH).powi(2)).exp() * 0.42;
    shape::mix(base, GLOW, haze)
}
