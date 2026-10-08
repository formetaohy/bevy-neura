use crate::display::Display;
use crate::terrain::{self, CHUNKS, PAD_X1, PAD_X2, PAD_Y, Terrain};
use bevy::math::primitives::ConvexPolygon;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;

const ROCK: Color = Color::srgb(0.13, 0.11, 0.17);
const CREST: Color = Color::srgb(0.23, 0.2, 0.31);
const RIDGE: Color = Color::srgb(0.47, 0.44, 0.62);
const CRATER: Color = Color::srgb(0.09, 0.08, 0.13);
const CRATER_RIM: Color = Color::srgb(0.3, 0.27, 0.4);
const PLATE: Color = Color::srgb(0.45, 0.48, 0.56);
const PLATE_EDGE: Color = Color::srgb(0.66, 0.7, 0.78);
const HAZARD: Color = Color::srgb(0.85, 0.63, 0.22);
const DARK_HAZARD: Color = Color::srgb(0.16, 0.17, 0.22);
const POLE: Color = Color::srgb(0.72, 0.75, 0.82);
const PENNANT: Color = Color::srgb(0.9, 0.75, 0.2);
const BEACON: Color = Color::srgb(0.5, 0.9, 1.0);
const CREST_DEPTH: f32 = 0.4;
const RIDGE_WIDTH: f32 = 0.07;
const CRATER_AT: [f32; 24] = [
    0.6, 1.3, 2.4, 3.1, 4.6, 5.2, 6.1, 7.3, 8.6, 9.4, 10.7, 11.6, 12.4, 13.7, 14.5, 15.3, 16.1,
    16.9, 17.4, 18.2, 18.9, 19.4, 2.9, 15.8,
];
const BEACON_AT: f32 = 0.42;
const BEACON_POLE: f32 = 0.5;
const FLAG_POLE: f32 = 1.7;
const PENNANT_LENGTH: f32 = 0.7;
const GROUND_LAYER: f32 = 1.0;

#[derive(Resource)]
pub struct Ground {
    generation: u64,
    parts: Vec<Entity>,
    beacons: [Entity; 2],
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    display: Res<Display>,
) {
    let beacons = [
        beacon(
            &mut commands,
            &mut meshes,
            &mut materials,
            PAD_X1 - BEACON_AT,
        ),
        beacon(
            &mut commands,
            &mut meshes,
            &mut materials,
            PAD_X2 + BEACON_AT,
        ),
    ];
    pad(&mut commands, &mut meshes, &mut materials);
    let mut ground = Ground {
        generation: 0,
        parts: Vec::new(),
        beacons,
    };
    build(
        &mut commands,
        &mut meshes,
        &mut materials,
        display.env().terrain(),
        &mut ground.parts,
    );
    ground.generation = display.env().generation();
    commands.insert_resource(ground);
}

pub fn refresh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    display: Res<Display>,
    mut ground: ResMut<Ground>,
) {
    if ground.generation == display.env().generation() {
        return;
    }
    for part in ground.parts.drain(..) {
        commands.entity(part).despawn();
    }
    build(
        &mut commands,
        &mut meshes,
        &mut materials,
        display.env().terrain(),
        &mut ground.parts,
    );
    ground.generation = display.env().generation();
}

pub fn pulse(time: Res<Time>, ground: Res<Ground>, mut sprites: Query<&mut Sprite>) {
    let wave = 0.5 + 0.5 * (time.elapsed_secs() * 2.6).sin();
    for beacon in ground.beacons {
        if let Ok(mut sprite) = sprites.get_mut(beacon) {
            sprite.color = BEACON.with_alpha(0.35 + 0.65 * wave);
        }
    }
}

fn build(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    terrain: &Terrain,
    parts: &mut Vec<Entity>,
) {
    let points = terrain.points();
    for index in 0..CHUNKS - 1 {
        let left = points[index];
        let right = points[index + 1];
        parts.push(quad(
            commands,
            meshes,
            materials,
            [left, right, Vec2::new(right.x, 0.0), Vec2::new(left.x, 0.0)],
            ROCK,
            0.1,
        ));
        parts.push(quad(
            commands,
            meshes,
            materials,
            [
                left,
                right,
                right - Vec2::new(0.0, CREST_DEPTH),
                left - Vec2::new(0.0, CREST_DEPTH),
            ],
            CREST,
            0.2,
        ));
        let segment = right - left;
        let length = segment.length();
        let middle = (left + right) / 2.0;
        parts.push(
            commands
                .spawn((
                    Sprite::from_color(RIDGE, Vec2::new(length, RIDGE_WIDTH)),
                    Transform::from_translation(
                        (middle + Vec2::new(0.0, RIDGE_WIDTH * 0.5)).extend(0.3),
                    )
                    .with_rotation(Quat::from_rotation_z(segment.y.atan2(segment.x))),
                ))
                .id(),
        );
    }
    for (index, at) in CRATER_AT.iter().enumerate() {
        let radius = if index % 3 == 0 { 0.34 } else { 0.2 };
        let on = terrain.closest(Vec2::new(*at, terrain::HEIGHT));
        let center = Vec2::new(*at, on.point.y);
        parts.push(
            commands
                .spawn((
                    Sprite::from_color(CRATER, Vec2::new(radius * 2.0, radius * 0.8)),
                    Transform::from_translation(
                        (center - Vec2::new(0.0, radius * 0.42)).extend(0.25),
                    ),
                ))
                .id(),
        );
        parts.push(
            commands
                .spawn((
                    Sprite::from_color(CRATER_RIM, Vec2::new(radius * 2.15, 0.045)),
                    Transform::from_translation(
                        (center + Vec2::new(0.0, radius * 0.02)).extend(0.26),
                    ),
                ))
                .id(),
        );
    }
}

fn pad(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) {
    let (low, high) = (PAD_X1, PAD_X2);
    quad(
        commands,
        meshes,
        materials,
        [
            Vec2::new(low - 0.25, PAD_Y - 0.16),
            Vec2::new(high + 0.25, PAD_Y - 0.16),
            Vec2::new(high + 0.25, PAD_Y + 0.14),
            Vec2::new(low - 0.25, PAD_Y + 0.14),
        ],
        PLATE,
        0.4,
    );
    commands.spawn((
        Sprite::from_color(PLATE_EDGE, Vec2::new(high - low + 0.5, 0.05)),
        Transform::from_xyz((low + high) / 2.0, PAD_Y + 0.15, 0.5),
    ));
    let stripes = 13;
    let width = (high - low) / stripes as f32;
    for stripe in 0..stripes {
        let color = if stripe % 2 == 0 { HAZARD } else { DARK_HAZARD };
        commands.spawn((
            Sprite::from_color(color, Vec2::new(width * 0.55, 0.07)),
            Transform::from_xyz(low + width * (stripe as f32 + 0.5), PAD_Y + 0.02, 0.45),
        ));
    }
    for x in [low, high] {
        commands.spawn((
            Sprite::from_color(POLE, Vec2::new(0.055, FLAG_POLE)),
            Transform::from_xyz(x, PAD_Y + FLAG_POLE / 2.0, 0.6),
        ));
        let pennant = Triangle2d::new(
            Vec2::new(0.0, 0.14),
            Vec2::new(0.0, -0.14),
            Vec2::new(PENNANT_LENGTH, 0.0),
        )
        .mesh()
        .build();
        commands.spawn((
            Mesh2d(meshes.add(pennant)),
            MeshMaterial2d(materials.add(ColorMaterial::from(PENNANT))),
            Transform::from_xyz(x + 0.03, PAD_Y + FLAG_POLE - 0.2, 0.6),
        ));
    }
}

fn beacon(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    at: f32,
) -> Entity {
    commands.spawn((
        Sprite::from_color(POLE, Vec2::new(0.05, BEACON_POLE)),
        Transform::from_xyz(at, PAD_Y + BEACON_POLE / 2.0, 0.6),
    ));
    let lamp = meshes.add(Circle::new(0.11).mesh().resolution(20).build());
    commands
        .spawn((
            Mesh2d(lamp),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: BEACON,
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(at, PAD_Y + BEACON_POLE, 0.7),
        ))
        .id()
}

fn quad(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    vertices: [Vec2; 4],
    color: Color,
    layer: f32,
) -> Entity {
    let mesh = ConvexPolygon::new(vertices.to_vec())
        .expect("a chunk of the moon holds a convex face")
        .mesh()
        .build();
    commands
        .spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_xyz(0.0, 0.0, GROUND_LAYER + layer),
        ))
        .id()
}
