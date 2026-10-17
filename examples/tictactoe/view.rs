use crate::game::{Cell, Game, Stage, Verdict};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{Display, InteractionDisabled};
use bevy_neura_example::theme::{BAD, DIM, GOOD, TITLE};
use bevy_neura_example::ui;

const HEADING: &str = "tic tac toe";
const CELL: f32 = 104.0;
const GAP: f32 = 8.0;
const BAR: f32 = 328.0;
const CELL_COLOR: Color = Color::srgb(0.07, 0.09, 0.14);
const HOVER_COLOR: Color = Color::srgb(0.11, 0.14, 0.21);
const PRESS_COLOR: Color = Color::srgb(0.16, 0.21, 0.31);
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
const PLAYER: Color = Color::srgb(0.40, 0.78, 1.0);
const COMPUTER: Color = Color::srgb(0.99, 0.62, 0.35);

#[derive(Component)]
pub(crate) struct Mark(usize);

#[derive(Component)]
pub(crate) struct Bar(Part);

#[derive(Clone, Copy)]
pub(crate) enum Part {
    Track,
    Fill,
}

#[derive(Component)]
pub(crate) struct Status;

type StatusText<'w, 's> =
    Query<'w, 's, (&'static mut Text, &'static mut TextColor), (With<Status>, Without<Mark>)>;

pub fn setup(mut commands: Commands) {
    commands
        .spawn((ui::screen(), Pickable::IGNORE))
        .with_children(|screen| {
            screen.spawn((Text::new(HEADING), ui::text(30.0, TITLE)));
            screen
                .spawn((
                    Node {
                        width: Val::Px(BAR),
                        height: Val::Px(10.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|wrapper| {
                    wrapper
                        .spawn((
                            Bar(Part::Track),
                            Node {
                                width: Val::Px(BAR),
                                height: Val::Px(4.0),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(TRACK),
                        ))
                        .with_children(|track| {
                            track.spawn((
                                Bar(Part::Fill),
                                Node {
                                    width: Val::Px(0.0),
                                    height: Val::Percent(100.0),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(FILL),
                            ));
                        });
                });
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(GAP),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|board| {
                    for row in 0..3 {
                        board
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(GAP),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                for column in 0..3 {
                                    let at = row * 3 + column;
                                    line.spawn((
                                        Button,
                                        Cell(at),
                                        Node {
                                            width: Val::Px(CELL),
                                            height: Val::Px(CELL),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            border_radius: BorderRadius::all(Val::Px(14.0)),
                                            ..default()
                                        },
                                        BackgroundColor(CELL_COLOR),
                                        children![(
                                            Mark(at),
                                            Text::default(),
                                            ui::text(64.0, TITLE),
                                        )],
                                    ));
                                }
                            });
                    }
                });
            screen.spawn((Status, Text::new("learning"), ui::text(17.0, DIM)));
        });
}

pub fn paint(
    game: Res<Game>,
    mut commands: Commands,
    mut cells: Query<(
        Entity,
        &Cell,
        &Interaction,
        &mut BackgroundColor,
        Has<InteractionDisabled>,
    )>,
    mut marks: Query<(&Mark, &mut Text, &mut TextColor), Without<Status>>,
    mut bars: Query<(&Bar, &mut Node)>,
    mut status: StatusText,
) {
    let board = game.board().cells();
    let human = game.stage() == Stage::Human;
    for (entity, cell, interaction, mut background, disabled) in &mut cells {
        if human && disabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
        if !human && !disabled {
            commands.entity(entity).insert(InteractionDisabled);
        }
        let empty = board[cell.0] == 0;
        background.0 = if human && empty {
            match interaction {
                Interaction::Pressed => PRESS_COLOR,
                Interaction::Hovered => HOVER_COLOR,
                Interaction::None => CELL_COLOR,
            }
        } else {
            CELL_COLOR
        };
    }
    for (mark, mut text, mut colour) in &mut marks {
        let (held, shade) = match board[mark.0] {
            1 => ("X", PLAYER),
            -1 => ("O", COMPUTER),
            _ => ("", TITLE),
        };
        if text.0 != held {
            text.0 = held.to_string();
        }
        if colour.0 != shade {
            colour.0 = shade;
        }
    }
    let learning = game.stage() == Stage::Learning;
    for (bar, mut node) in &mut bars {
        match bar.0 {
            Part::Track => {
                let display = if learning {
                    Display::Flex
                } else {
                    Display::None
                };
                if node.display != display {
                    node.display = display;
                }
            }
            Part::Fill => node.width = Val::Px(BAR * game.progress()),
        }
    }
    let (held, shade) = match game.stage() {
        Stage::Done(Verdict::Win) => (game.message(), GOOD),
        Stage::Done(Verdict::Draw) => (game.message(), TITLE),
        Stage::Done(Verdict::Loss) => (game.message(), BAD),
        Stage::Learning | Stage::Human | Stage::Computer => (game.message(), DIM),
    };
    for (mut text, mut colour) in &mut status {
        if text.0 != held {
            text.0 = held.to_string();
        }
        if colour.0 != shade {
            colour.0 = shade;
        }
    }
}
