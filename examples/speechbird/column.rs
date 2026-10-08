use crate::pipe::{self, Pipe};
use crate::shape;
use crate::view;
use bevy::prelude::*;

const HALF: f32 = pipe::WIDTH / 2.0;
const COLUMNS: usize = 44;
const CORNER: f32 = 15.0;
const CAP: f32 = 28.0;
const RIM: f32 = 3.0;
const REACH: f32 = crate::world::HEIGHT;
const OUTLINE: f32 = 3.0;
const SHADE: [f32; 4] = [1.14, 0.96, 0.42, 0.80];
const OUTLINE_COLOR: Color = Color::srgb(0.025, 0.04, 0.07);
const PROFILE: [(f32, Color); 5] = [
    (0.0, Color::srgb(0.06, 0.14, 0.10)),
    (0.09, Color::srgb(0.17, 0.38, 0.23)),
    (0.34, Color::srgb(0.40, 0.70, 0.44)),
    (0.62, Color::srgb(0.19, 0.44, 0.27)),
    (1.0, Color::srgb(0.06, 0.14, 0.10)),
];

#[derive(Component)]
pub struct Dressed;

#[derive(Resource)]
pub struct Art {
    column: Handle<Mesh>,
    outline: Handle<Mesh>,
    look: Handle<ColorMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Art {
        column: meshes.add(column(HALF, CORNER, CAP, REACH, false)),
        outline: meshes.add(column(
            HALF + OUTLINE,
            CORNER + OUTLINE,
            CAP,
            REACH + OUTLINE,
            true,
        )),
        look: materials.add(shape::blend(Color::WHITE)),
    });
}

pub fn dress(
    mut commands: Commands,
    art: Res<Art>,
    mut pipes: Query<(Entity, &mut Transform, &Pipe), Without<Dressed>>,
) {
    for (pipe, mut place, frame) in &mut pipes {
        place.translation.z = view::PIPE;
        let lower = frame.gap - pipe::GAP / 2.0;
        let upper = frame.gap + pipe::GAP / 2.0;
        commands
            .entity(pipe)
            .insert(Dressed)
            .with_children(|columns| {
                for (at, flip) in [(lower + OUTLINE, 1.0), (upper - OUTLINE, -1.0)] {
                    columns.spawn((
                        Mesh2d(art.outline.clone()),
                        MeshMaterial2d(art.look.clone()),
                        Transform::from_xyz(0.0, at, 0.0).with_scale(Vec3::new(1.0, flip, 1.0)),
                    ));
                }
                for (at, flip) in [(lower, 1.0), (upper, -1.0)] {
                    columns.spawn((
                        Mesh2d(art.column.clone()),
                        MeshMaterial2d(art.look.clone()),
                        Transform::from_xyz(0.0, at, 0.05).with_scale(Vec3::new(1.0, flip, 1.0)),
                    ));
                }
            });
    }
}

fn column(half: f32, corner: f32, cap: f32, reach: f32, outline: bool) -> Mesh {
    let rows = [0.0, -RIM, -cap, -reach];
    let points = rows
        .iter()
        .map(|at| {
            (0..COLUMNS)
                .map(|column| {
                    let x = -half + 2.0 * half * column as f32 / (COLUMNS - 1) as f32;
                    Vec2::new(
                        x,
                        at - corner_inset(x, half, corner) * corner_reach(*at, corner),
                    )
                })
                .collect::<Vec<Vec2>>()
        })
        .collect::<Vec<Vec<Vec2>>>();
    shape::grid(&points, |row, column, _| {
        if outline {
            OUTLINE_COLOR
        } else {
            let share = column as f32 / (COLUMNS - 1) as f32;
            shape::gain(shape::ramp(&PROFILE, share), SHADE[row])
        }
    })
}

fn corner_inset(x: f32, half: f32, corner: f32) -> f32 {
    let inset = (x.abs() - (half - corner)).clamp(0.0, corner);
    corner - (corner * corner - inset * inset).sqrt()
}

fn corner_reach(at: f32, corner: f32) -> f32 {
    1.0 - (-at / corner).clamp(0.0, 1.0)
}
