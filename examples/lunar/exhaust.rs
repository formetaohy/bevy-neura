use crate::display::Display;
use crate::terrain::SCALE;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;

const POOL: usize = 56;
const RADIUS: f32 = 2.0 / SCALE;
const EXHAUST_LAYER: f32 = 4.0;

#[derive(Resource)]
pub struct Exhaust {
    puffs: Vec<(Entity, Handle<ColorMaterial>)>,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puff = meshes.add(Circle::new(RADIUS).mesh().resolution(16).build());
    let mut puffs = Vec::new();
    for _ in 0..POOL {
        let material = materials.add(ColorMaterial {
            color: Color::srgba(1.0, 0.5, 0.5, 0.9),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        });
        let entity = commands
            .spawn((
                Mesh2d(puff.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(0.0, 0.0, EXHAUST_LAYER),
                Visibility::Hidden,
            ))
            .id();
        puffs.push((entity, material));
    }
    commands.insert_resource(Exhaust { puffs });
}

pub fn paint(
    display: Res<Display>,
    exhaust: Res<Exhaust>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut transforms: Query<&mut Transform>,
    mut places: Query<&mut Visibility>,
) {
    let particles = display.env().particles().iter();
    let mut shown = 0;
    for (body, ttl) in particles {
        if shown >= POOL {
            break;
        }
        let (entity, material) = &exhaust.puffs[shown];
        if let Ok(mut place) = transforms.get_mut(*entity) {
            place.translation = body.center.extend(EXHAUST_LAYER);
        }
        if let Ok(mut place) = places.get_mut(*entity) {
            *place = Visibility::Inherited;
        }
        if let Some(mut material) = materials.get_mut(material) {
            let hot = (0.15 + ttl).clamp(0.2, 1.0);
            let dim = (0.5 * ttl).max(0.2);
            material.color = Color::srgba(hot, dim, dim, 0.85);
        }
        shown += 1;
    }
    for (entity, _) in exhaust.puffs.iter().skip(shown) {
        if let Ok(mut place) = places.get_mut(*entity) {
            *place = Visibility::Hidden;
        }
    }
}
