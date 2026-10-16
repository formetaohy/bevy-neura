use crate::canvas;
use crate::game::Game;
use bevy::asset::RenderAssetUsages;
use bevy::picking::Pickable;
use bevy::prelude::*;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

const BACKGROUND: Color = Color::srgb(0.035, 0.045, 0.065);
const TITLE: Color = Color::srgb(0.94, 0.97, 1.0);
const DIM: Color = Color::srgb(0.55, 0.62, 0.76);
const RIGHT: Color = Color::srgb(0.42, 0.85, 0.52);
const WRONG: Color = Color::srgb(0.95, 0.42, 0.42);
const PAPER: [f32; 3] = [0.95, 0.95, 0.92];
const INK: [f32; 3] = [0.07, 0.08, 0.12];
const HINT: &str = "write the answer in the box and the network reads it";

#[derive(Component)]
pub struct CanvasNode;

#[derive(Component)]
pub(crate) struct Line(Part);

pub(crate) enum Part {
    Expression,
    Reading,
    Score,
}

#[derive(Resource)]
pub(crate) struct Paper {
    handle: Handle<Image>,
    painted: u32,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(BACKGROUND));
    commands.spawn(Camera2d);
}

pub fn board(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let side = canvas::SIDE as u32;
    let size = (side * side) as usize;
    let handle = images.add(Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; size * 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    commands.insert_resource(Paper {
        handle: handle.clone(),
        painted: u32::MAX,
    });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|screen| {
            screen.spawn((
                Text::new(HINT),
                TextFont::from_font_size(17.0),
                TextColor(DIM),
            ));
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(26.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        Line(Part::Expression),
                        Text::default(),
                        TextFont::from_font_size(46.0),
                        TextColor(TITLE),
                    ));
                    row.spawn((
                        CanvasNode,
                        Node {
                            width: Val::Px(canvas::SIDE),
                            height: Val::Px(canvas::SIDE),
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ));
                });
            screen
                .spawn((
                    Node {
                        height: Val::Px(36.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|line| {
                    line.spawn((
                        Line(Part::Reading),
                        Text::default(),
                        TextFont::from_font_size(24.0),
                        TextColor(DIM),
                    ));
                });
            screen.spawn((
                Line(Part::Score),
                Text::default(),
                TextFont::from_font_size(16.0),
                TextColor(DIM),
            ));
        });
}

pub fn paint(
    game: Res<Game>,
    mut paper: ResMut<Paper>,
    mut images: ResMut<Assets<Image>>,
    mut lines: Query<(&Line, &mut Text, &mut TextColor)>,
) {
    if paper.painted != game.revision() {
        let mut paper_image = images
            .get_mut(&paper.handle)
            .expect("the board holds the paper it draws");
        let pixels = paper_image
            .data
            .as_mut()
            .expect("the paper holds the pixels it paints");
        for (pixel, ink) in pixels
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(game.ink().values())
        {
            for at in 0..3 {
                pixel[at] = ((PAPER[at] + (INK[at] - PAPER[at]) * ink) * 255.0) as u8;
            }
            pixel[3] = 255;
        }
        paper.painted = game.revision();
    }
    for (line, mut text, mut colour) in &mut lines {
        match line.0 {
            Part::Expression => write(&mut text, game.expression(), TITLE, &mut colour),
            Part::Reading => {
                let (held, shade) = match (game.reading(), game.verdict()) {
                    (Some(guess), Some(true)) => (
                        format!(
                            "the network reads {} ({:.0}%) - right",
                            guess.digit,
                            guess.confidence * 100.0,
                        ),
                        RIGHT,
                    ),
                    (Some(guess), Some(false)) => (
                        format!(
                            "the network reads {} ({:.0}%) - wrong",
                            guess.digit,
                            guess.confidence * 100.0,
                        ),
                        WRONG,
                    ),
                    _ => (String::new(), DIM),
                };
                write(&mut text, held, shade, &mut colour);
            }
            Part::Score => write(
                &mut text,
                format!("right {} - missed {}", game.right(), game.missed()),
                DIM,
                &mut colour,
            ),
        }
    }
}

fn write(text: &mut Text, held: String, shade: Color, colour: &mut TextColor) {
    if text.0 != held {
        text.0 = held;
    }
    if colour.0 != shade {
        colour.0 = shade;
    }
}
