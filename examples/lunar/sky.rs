use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use std::f32::consts::TAU;

const ROWS: u32 = 72;
const GLOW: (f32, f32, f32) = (0.34, 0.19, 0.32);
const STARS: u32 = 260;
const FLOOR: f32 = -6.0;
const CEILING: f32 = 22.0;
const WIDTH: f32 = 72.0;
const NIGHT: (f32, f32, f32) = (0.01, 0.012, 0.03);
const HAZE: (f32, f32, f32) = (0.13, 0.06, 0.18);
const EARTH: Vec2 = Vec2::new(17.4, 15.8);
const EARTH_RADIUS: f32 = 3.6;
const OCEAN: (f32, f32, f32) = (0.09, 0.22, 0.43);
const ATMOSPHERE: (f32, f32, f32) = (0.3, 0.62, 0.95);
const NIGHT_SIDE: (f32, f32, f32) = (0.02, 0.03, 0.08);
const LAND: (f32, f32, f32) = (0.2, 0.44, 0.32);
const STAR_TINTS: [(f32, f32, f32); 4] = [
    (0.95, 0.96, 1.0),
    (0.78, 0.85, 1.0),
    (1.0, 0.9, 0.78),
    (0.9, 0.94, 0.9),
];
const BAND_LAYER: f32 = -20.0;
const STAR_LAYER: f32 = -18.0;
const EARTH_LAYER: f32 = -19.0;

#[derive(Component)]
pub(crate) struct Star {
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
        Mesh2d(meshes.add(gradient())),
        MeshMaterial2d(materials.add(blend(Color::WHITE))),
        Transform::from_xyz(10.0, 0.0, BAND_LAYER),
    ));
    let mut random = crate::random::Random::seeded(0x5c17_e510);
    for _ in 0..STARS {
        let tint = STAR_TINTS[random.below(STAR_TINTS.len())];
        commands.spawn((
            Sprite::from_color(
                srgb(tint).with_alpha(random.uniform(0.35, 1.0)),
                Vec2::splat(random.uniform(0.025, 0.08)),
            ),
            Transform::from_xyz(
                random.uniform(-6.0, 26.0),
                random.uniform(FLOOR + 1.0, CEILING),
                STAR_LAYER,
            )
            .with_rotation(Quat::from_rotation_z(TAU * random.unit() as f32)),
            Star {
                tint: srgb(tint),
                phase: TAU * random.unit() as f32,
                speed: random.uniform(0.5, 2.2),
            },
        ));
    }
    let disc = meshes.add(Circle::new(EARTH_RADIUS).mesh().resolution(96).build());
    commands.spawn((
        Mesh2d(disc.clone()),
        MeshMaterial2d(materials.add(blend(srgb(OCEAN)))),
        Transform::from_translation(EARTH.extend(EARTH_LAYER)),
    ));
    let halo = meshes.add(
        Circle::new(EARTH_RADIUS * 1.06)
            .mesh()
            .resolution(96)
            .build(),
    );
    commands.spawn((
        Mesh2d(halo),
        MeshMaterial2d(materials.add(blend(srgb(ATMOSPHERE).with_alpha(0.24)))),
        Transform::from_translation((EARTH + Vec2::new(-0.08, 0.08)).extend(EARTH_LAYER - 0.1)),
    ));
    let shade = meshes.add(Circle::new(EARTH_RADIUS).mesh().resolution(96).build());
    commands.spawn((
        Mesh2d(shade),
        MeshMaterial2d(materials.add(blend(srgb(NIGHT_SIDE)))),
        Transform::from_translation((EARTH + Vec2::new(1.25, -1.45)).extend(EARTH_LAYER + 0.1)),
    ));
    let land = meshes.add(Circle::new(0.5).mesh().resolution(32).build());
    for (at, shape) in [
        (Vec2::new(-1.15, 1.0), Vec2::new(1.2, 0.55)),
        (Vec2::new(0.95, 0.15), Vec2::new(0.7, 0.95)),
        (Vec2::new(-0.35, -1.1), Vec2::new(1.4, 0.45)),
        (Vec2::new(-1.9, -0.45), Vec2::new(0.5, 0.7)),
    ] {
        commands.spawn((
            Mesh2d(land.clone()),
            MeshMaterial2d(materials.add(blend(srgb(LAND)))),
            Transform::from_translation((EARTH + at).extend(EARTH_LAYER + 0.05))
                .with_scale(shape.extend(1.0)),
        ));
    }
}

fn gradient() -> Mesh {
    let row = (CEILING - FLOOR) / ROWS as f32;
    let (mut positions, mut colors, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for step in 0..=ROWS {
        let y = FLOOR + step as f32 * row;
        let color = sky_at(y).to_linear();
        for x in [-WIDTH / 2.0, WIDTH / 2.0] {
            positions.push([x, y, 0.0]);
            colors.push([color.red, color.green, color.blue, 1.0]);
        }
    }
    for step in 0..ROWS {
        let low = step * 2;
        indices.extend_from_slice(&[low, low + 1, low + 3, low, low + 3, low + 2]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn sky_at(y: f32) -> Color {
    let from_top = ((CEILING - y) / (CEILING - FLOOR)).clamp(0.0, 1.0);
    let base = mix(NIGHT, HAZE, from_top.powi(3));
    let glow = (-((y - crate::terrain::PAD_Y) / 1.9).powi(2)).exp() * 0.55;
    mix_color(base, srgb(GLOW), glow)
}

fn mix_color(low: Color, high: Color, share: f32) -> Color {
    let low = low.to_linear();
    let high = high.to_linear();
    Color::linear_rgb(
        low.red + (high.red - low.red) * share,
        low.green + (high.green - low.green) * share,
        low.blue + (high.blue - low.blue) * share,
    )
}

pub fn twinkle(time: Res<Time>, mut stars: Query<(&Star, &mut Sprite)>) {
    let seconds = time.elapsed_secs();
    for (star, mut sprite) in &mut stars {
        let wave = 0.5 + 0.5 * (seconds * star.speed + star.phase).sin();
        sprite.color = star.tint.with_alpha(0.35 + 0.65 * wave);
    }
}

fn srgb(color: (f32, f32, f32)) -> Color {
    Color::srgb(color.0, color.1, color.2)
}

fn mix(low: (f32, f32, f32), high: (f32, f32, f32), share: f32) -> Color {
    Color::srgb(
        low.0 + (high.0 - low.0) * share,
        low.1 + (high.1 - low.1) * share,
        low.2 + (high.2 - low.2) * share,
    )
}

fn blend(color: Color) -> ColorMaterial {
    ColorMaterial {
        color,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    }
}
