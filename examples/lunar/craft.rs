use crate::body::{Body, HULL, LEFT_LEG, RIGHT_LEG};
use crate::display::Display;
use crate::env::Action;
use bevy::math::primitives::ConvexPolygon;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;

const HULL_FILL: Color = Color::srgb(0.52, 0.46, 0.86);
const HULL_EDGE: Color = Color::srgb(0.13, 0.11, 0.26);
const HULL_TRIM: Color = Color::srgb(0.85, 0.87, 0.95);
const WINDOW: Color = Color::srgb(1.0, 0.79, 0.34);
const LEG_FILL: Color = Color::srgb(0.44, 0.42, 0.64);
const LEG_FOOT: Color = Color::srgb(0.78, 0.8, 0.9);
const FLAME_CORE: Color = Color::srgb(1.0, 0.98, 0.86);
const FLAME_SKIN: Color = Color::srgb(1.0, 0.62, 0.18);
const FLAME_GLOW: Color = Color::srgb(1.0, 0.5, 0.12);
const WINDOW_AT: Vec2 = Vec2::new(0.0, 0.3);
const WINDOW_RADIUS: f32 = 0.12;
const NOZZLE_AT: Vec2 = Vec2::new(0.0, -0.42);
const PLUME: f32 = 0.8;
const THRUSTER_AT: f32 = 0.5;
const THRUSTER_Y: f32 = 0.04;
const THRUSTER_LENGTH: f32 = 0.44;
const CRAFT_LAYER: f32 = 6.0;

#[derive(Resource)]
pub struct Craft {
    root: Entity,
    plume: Entity,
    plume_core: Entity,
    plume_glow: Entity,
    positive: Entity,
    negative: Entity,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    display: Res<Display>,
) {
    let env = display.env();
    let body = env.craft();
    let root = commands
        .spawn((
            Transform::from_translation(body.origin().extend(CRAFT_LAYER)),
            Visibility::default(),
        ))
        .id();
    let hull = local(body, HULL);
    fill(
        &mut commands,
        &mut meshes,
        &mut materials,
        root,
        hull.iter().map(|vertex| *vertex * 1.09).collect(),
        HULL_EDGE,
        0.1,
    );
    fill(
        &mut commands,
        &mut meshes,
        &mut materials,
        root,
        hull,
        HULL_FILL,
        0.2,
    );
    for (at, size, color) in [
        (Vec2::new(0.0, 0.06), Vec2::new(0.66, 0.05), HULL_TRIM),
        (Vec2::new(0.0, -0.14), Vec2::new(0.76, 0.03), HULL_EDGE),
        (NOZZLE_AT, Vec2::new(0.3, 0.1), HULL_EDGE),
    ] {
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_translation(at.extend(0.3)),
            ChildOf(root),
        ));
    }
    let window = meshes.add(Circle::new(WINDOW_RADIUS).mesh().resolution(24).build());
    commands.spawn((
        Mesh2d(window),
        MeshMaterial2d(materials.add(blend(WINDOW))),
        Transform::from_translation(WINDOW_AT.extend(0.35)),
        ChildOf(root),
    ));
    for tag in [RIGHT_LEG, LEFT_LEG] {
        let leg = local(body, tag);
        fill(
            &mut commands,
            &mut meshes,
            &mut materials,
            root,
            leg.iter().map(|vertex| *vertex * 1.08).collect(),
            HULL_EDGE,
            0.15,
        );
        fill(
            &mut commands,
            &mut meshes,
            &mut materials,
            root,
            leg.clone(),
            LEG_FILL,
            0.25,
        );
        let mut lowest = leg[0];
        let mut second = leg[0];
        for vertex in &leg {
            if vertex.y < lowest.y {
                second = lowest;
                lowest = *vertex;
            } else if vertex.y < second.y {
                second = *vertex;
            }
        }
        let foot = (lowest + second) / 2.0;
        commands.spawn((
            Sprite::from_color(LEG_FOOT, Vec2::new(0.17, 0.05)),
            Transform::from_translation((foot + Vec2::new(0.0, -0.02)).extend(0.35)),
            ChildOf(root),
        ));
    }
    let plume = meshes.add(cone(0.12, PLUME));
    let core = meshes.add(cone(0.055, PLUME * 0.5));
    let glow = meshes.add(Circle::new(0.24).mesh().resolution(24).build());
    let plume = commands
        .spawn((
            Mesh2d(plume),
            MeshMaterial2d(materials.add(blend(FLAME_SKIN.with_alpha(0.85)))),
            Transform::from_translation((NOZZLE_AT + Vec2::new(0.0, -0.02)).extend(0.5)),
            ChildOf(root),
        ))
        .id();
    let plume_core = commands
        .spawn((
            Mesh2d(core),
            MeshMaterial2d(materials.add(blend(FLAME_CORE))),
            Transform::from_translation((NOZZLE_AT + Vec2::new(0.0, -0.02)).extend(0.6)),
            ChildOf(root),
        ))
        .id();
    let plume_glow = commands
        .spawn((
            Mesh2d(glow),
            MeshMaterial2d(materials.add(blend(FLAME_GLOW.with_alpha(0.3)))),
            Transform::from_translation((NOZZLE_AT + Vec2::new(0.0, -0.26)).extend(0.4)),
            ChildOf(root),
        ))
        .id();
    let jet = meshes.add(
        cone(0.07, THRUSTER_LENGTH).rotated_by(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
    );
    let positive = commands
        .spawn((
            Mesh2d(jet.clone()),
            MeshMaterial2d(materials.add(blend(FLAME_SKIN.with_alpha(0.85)))),
            Transform::from_translation(Vec2::new(THRUSTER_AT, THRUSTER_Y).extend(0.5)),
            ChildOf(root),
        ))
        .id();
    let negative = commands
        .spawn((
            Mesh2d(jet),
            MeshMaterial2d(materials.add(blend(FLAME_SKIN.with_alpha(0.85)))),
            Transform::from_translation(Vec2::new(-THRUSTER_AT, THRUSTER_Y).extend(0.5)),
            ChildOf(root),
        ))
        .id();
    commands.insert_resource(Craft {
        root,
        plume,
        plume_core,
        plume_glow,
        positive,
        negative,
    });
}

pub fn paint(
    time: Res<Time>,
    display: Res<Display>,
    craft: Res<Craft>,
    mut transforms: Query<&mut Transform>,
    mut places: Query<&mut Visibility>,
) {
    let env = display.env();
    let body = env.craft();
    if let Ok(mut place) = transforms.get_mut(craft.root) {
        place.translation = body.origin().extend(CRAFT_LAYER);
        place.rotation = Quat::from_rotation_z(body.angle);
    }
    let (main, lateral) = env.thrust();
    let flicker = 0.8 + 0.2 * (time.elapsed_secs() * 43.0).sin();
    let firing = main > 0.0;
    for entity in [craft.plume, craft.plume_core, craft.plume_glow] {
        show(&mut places, entity, firing);
    }
    stretch(
        &mut transforms,
        craft.plume,
        Vec2::new(1.0, flicker * main.max(0.0)),
    );
    stretch(
        &mut transforms,
        craft.plume_core,
        Vec2::new(0.92 + 0.16 * flicker, flicker * main.max(0.0)),
    );
    let side = lateral > 0.0;
    show(
        &mut places,
        craft.positive,
        side && env.action() == Action::Left,
    );
    show(
        &mut places,
        craft.negative,
        side && env.action() == Action::Right,
    );
    for entity in [craft.positive, craft.negative] {
        stretch(&mut transforms, entity, Vec2::splat(0.75 + 0.4 * flicker));
    }
}

fn local(body: &Body, tag: u8) -> Vec<Vec2> {
    body.part(tag)
        .polygon
        .vertices()
        .iter()
        .map(|vertex| *vertex - body.local_center)
        .collect()
}

fn cone(radius: f32, length: f32) -> Mesh {
    Triangle2d::new(
        Vec2::new(-radius, 0.0),
        Vec2::new(radius, 0.0),
        Vec2::new(0.0, -length),
    )
    .mesh()
    .build()
}

fn show(places: &mut Query<&mut Visibility>, entity: Entity, shown: bool) {
    if let Ok(mut place) = places.get_mut(entity) {
        *place = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn stretch(transforms: &mut Query<&mut Transform>, entity: Entity, size: Vec2) {
    if let Ok(mut place) = transforms.get_mut(entity) {
        place.scale = size.extend(1.0);
    }
}

fn fill(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    parent: Entity,
    vertices: Vec<Vec2>,
    color: Color,
    layer: f32,
) {
    let mesh = ConvexPolygon::new(vertices)
        .expect("the lander of the moon holds convex parts")
        .mesh()
        .build();
    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(blend(color))),
        Transform::from_xyz(0.0, 0.0, layer),
        ChildOf(parent),
    ));
}

fn blend(color: Color) -> ColorMaterial {
    ColorMaterial {
        color,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    }
}
