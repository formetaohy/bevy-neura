use crate::bird::{self, Bird};
use crate::game::Wingbeat;
use crate::shape;
use crate::view;
use bevy::prelude::*;

const BEAT: f32 = 0.42;
const SQUASH: f32 = 0.09;
const RAISED: f32 = -0.84;
const DOWN: f32 = 0.68;
const REST: f32 = -0.16;
const LIGHT: Vec2 = Vec2::new(-0.6, 0.8);
const OUTLINE: Color = Color::srgb(0.05, 0.06, 0.11);
const BODY_LIGHT: Color = Color::srgb(1.0, 0.91, 0.47);
const BODY_DARK: Color = Color::srgb(0.87, 0.55, 0.14);
const WING_LIGHT: Color = Color::srgb(0.99, 0.76, 0.28);
const WING_DARK: Color = Color::srgb(0.70, 0.42, 0.09);
const BELLY: Color = Color::srgb(1.0, 0.96, 0.78);
const CHEEK: Color = Color::srgb(1.0, 0.52, 0.50);
const EYE_WHITE: Color = Color::srgb(1.0, 0.99, 0.97);
const PUPIL: Color = Color::srgb(0.05, 0.06, 0.10);
const TAIL: Color = Color::srgb(0.80, 0.45, 0.12);
const CREST: Color = Color::srgb(0.93, 0.62, 0.16);
const BEAK_UPPER: Color = Color::srgb(1.0, 0.68, 0.22);
const BEAK_LOWER: Color = Color::srgb(0.85, 0.42, 0.12);
const AURA: Color = Color::srgb(1.0, 0.80, 0.42);

#[derive(Component, Default)]
pub struct Wing {
    phase: f32,
}

type Unplumed<'w, 's> = Query<'w, 's, (Entity, &'static mut Transform), Added<Bird>>;
type Wings<'w, 's> = Query<'w, 's, (&'static mut Wing, &'static mut Transform), Without<Bird>>;

pub fn dress(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut birds: Unplumed,
) {
    let Ok((bird, mut place)) = birds.single_mut() else {
        return;
    };
    let aura = meshes.add(shape::disc(34.0, 5, 44, |offset| {
        AURA.with_alpha(0.16 * (1.0 - (offset.length() / 34.0).clamp(0.0, 1.0)).powf(2.6))
    }));
    let tail = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 7.0),
        Vec2::new(0.0, -7.0),
        Vec2::new(-17.0, 1.0),
    ));
    let crest = meshes.add(Triangle2d::new(
        Vec2::new(-3.4, -5.0),
        Vec2::new(3.4, -5.0),
        Vec2::new(-0.8, 6.4),
    ));
    let outline = meshes.add(Circle::new(bird::RADIUS + 1.9));
    let body = meshes.add(shape::disc(bird::RADIUS, 10, 48, |offset| {
        shape::ball(offset, bird::RADIUS, LIGHT, BODY_DARK, BODY_LIGHT)
    }));
    let belly = meshes.add(shape::disc(11.0, 5, 32, |offset| {
        BELLY.with_alpha(0.32 * (1.0 - (offset.length() / 11.0).clamp(0.0, 1.0)).powf(1.6))
    }));
    let cheek = meshes.add(shape::disc(5.0, 4, 24, |offset| {
        CHEEK.with_alpha(0.30 * (1.0 - (offset.length() / 5.0).clamp(0.0, 1.0)).powf(1.4))
    }));
    let wing = meshes.add(shape::disc(11.0, 5, 32, |offset| {
        shape::ball(offset, 11.0, LIGHT, WING_DARK, WING_LIGHT)
    }));
    let eye = meshes.add(Circle::new(5.6));
    let white = meshes.add(Circle::new(4.7));
    let pupil = meshes.add(Circle::new(2.4));
    let glint = meshes.add(Circle::new(1.05));
    let beak_upper = meshes.add(Triangle2d::new(
        Vec2::new(5.0, 4.8),
        Vec2::new(5.0, 0.7),
        Vec2::new(19.4, 1.5),
    ));
    let beak_lower = meshes.add(Triangle2d::new(
        Vec2::new(5.0, -0.7),
        Vec2::new(5.0, -4.6),
        Vec2::new(16.4, 0.3),
    ));
    let flat = |materials: &mut Assets<ColorMaterial>, color: Color| {
        materials.add(ColorMaterial::from(color))
    };
    let soft =
        |materials: &mut Assets<ColorMaterial>, color: Color| materials.add(shape::blend(color));
    let aura_look = soft(&mut materials, Color::WHITE);
    let tail_look = flat(&mut materials, TAIL);
    let crest_look = flat(&mut materials, CREST);
    let outline_look = flat(&mut materials, OUTLINE);
    let body_look = flat(&mut materials, Color::WHITE);
    let belly_look = soft(&mut materials, Color::WHITE);
    let cheek_look = soft(&mut materials, Color::WHITE);
    let wing_look = flat(&mut materials, Color::WHITE);
    let eye_look = flat(&mut materials, OUTLINE);
    let white_look = flat(&mut materials, EYE_WHITE);
    let pupil_look = flat(&mut materials, PUPIL);
    let beak_upper_look = flat(&mut materials, BEAK_UPPER);
    let beak_lower_look = flat(&mut materials, BEAK_LOWER);
    place.translation.z = view::BIRD;
    commands.entity(bird).with_children(|bird| {
        bird.spawn((
            Mesh2d(aura),
            MeshMaterial2d(aura_look.clone()),
            Transform::from_xyz(0.0, 0.0, -0.7),
        ));
        bird.spawn((
            Mesh2d(tail),
            MeshMaterial2d(tail_look.clone()),
            Transform::from_xyz(-13.0, -1.0, -0.5),
        ));
        bird.spawn((
            Mesh2d(crest.clone()),
            MeshMaterial2d(crest_look.clone()),
            Transform::from_xyz(0.6, 13.6, -0.4).with_rotation(Quat::from_rotation_z(-0.30)),
        ));
        bird.spawn((
            Mesh2d(crest),
            MeshMaterial2d(crest_look),
            Transform::from_xyz(-4.6, 13.0, -0.4).with_rotation(Quat::from_rotation_z(-0.72)),
        ));
        bird.spawn((
            Mesh2d(outline),
            MeshMaterial2d(outline_look.clone()),
            Transform::from_xyz(0.0, 0.0, -0.3),
        ));
        bird.spawn((
            Wing { phase: 1.0 },
            Visibility::Inherited,
            Transform::from_xyz(-3.0, -1.5, -0.2),
        ))
        .with_children(|pinion| {
            pinion.spawn((
                Mesh2d(wing),
                MeshMaterial2d(wing_look.clone()),
                Transform::from_xyz(-9.5, -0.5, 0.0).with_scale(Vec3::new(1.0, 0.62, 1.0)),
            ));
        });
        bird.spawn((
            Mesh2d(body),
            MeshMaterial2d(body_look.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        bird.spawn((
            Mesh2d(belly),
            MeshMaterial2d(belly_look.clone()),
            Transform::from_xyz(3.0, -5.5, 0.1),
        ));
        bird.spawn((
            Mesh2d(cheek),
            MeshMaterial2d(cheek_look.clone()),
            Transform::from_xyz(5.5, -1.5, 0.12),
        ));
        bird.spawn((
            Mesh2d(eye),
            MeshMaterial2d(eye_look.clone()),
            Transform::from_xyz(4.6, 6.2, 0.2),
        ));
        bird.spawn((
            Mesh2d(white),
            MeshMaterial2d(white_look.clone()),
            Transform::from_xyz(4.6, 6.2, 0.21),
        ));
        bird.spawn((
            Mesh2d(pupil),
            MeshMaterial2d(pupil_look),
            Transform::from_xyz(6.2, 5.8, 0.22),
        ));
        bird.spawn((
            Mesh2d(glint),
            MeshMaterial2d(white_look),
            Transform::from_xyz(5.6, 7.5, 0.23),
        ));
        bird.spawn((
            Mesh2d(beak_lower),
            MeshMaterial2d(beak_lower_look),
            Transform::from_xyz(0.0, 0.0, 0.24),
        ));
        bird.spawn((
            Mesh2d(beak_upper),
            MeshMaterial2d(beak_upper_look),
            Transform::from_xyz(0.0, 0.0, 0.25),
        ));
    });
}

pub fn animate(
    time: Res<Time>,
    mut beats: MessageReader<Wingbeat>,
    mut birds: Query<(&Bird, &mut Transform, &Children), Without<Wing>>,
    mut wings: Wings,
) {
    let seconds = time.delta_secs();
    let mut flapped = false;
    for _ in beats.read() {
        flapped = true;
    }
    for (_, mut place, children) in &mut birds {
        for child in children {
            let Ok((mut wing, mut at)) = wings.get_mut(*child) else {
                continue;
            };
            if flapped {
                wing.phase = 0.0;
            }
            wing.phase = (wing.phase + seconds / BEAT).min(1.0);
            let stroke = stroke(wing.phase);
            at.rotation = Quat::from_rotation_z(stroke);
            at.translation.z = if stroke > 0.12 { 0.2 } else { -0.2 };
            let squash = SQUASH * (1.0 - (wing.phase / 0.35).min(1.0)).powi(2);
            place.scale = Vec3::new(1.0 + squash, 1.0 - squash, 1.0);
        }
    }
}

fn stroke(share: f32) -> f32 {
    if share < 0.20 {
        ease(RAISED, DOWN, share / 0.20)
    } else if share < 0.62 {
        ease(DOWN, REST, (share - 0.20) / 0.42)
    } else {
        REST
    }
}

fn ease(low: f32, high: f32, share: f32) -> f32 {
    low + (high - low) * share * share * (3.0 - 2.0 * share)
}
