use crate::board::Ending;
use crate::game::{Cell, Game, Stage, Verdict};
use crate::results::{CHUNK, RECENT, Results};
use crate::training::TOTAL_GAMES;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{Display, InteractionDisabled};
use bevy_neura_example::theme::{BAD, DIM, GOOD, TITLE};
use bevy_neura_example::ui;

const HEADING: &str = "tic tac toe";
const TILE: f32 = 104.0;
const GAP: f32 = 8.0;
const BOARD: f32 = 3.0 * TILE + 2.0 * GAP;
const PANEL: f32 = 408.0;
const CONTENT: f32 = 384.0;
const CHART: f32 = 54.0;
const RECENT_STRIP: f32 = 20.0;
const REFRESH: f32 = 0.1;
const CELL_COLOR: Color = Color::srgb(0.07, 0.09, 0.14);
const HOVER_COLOR: Color = Color::srgb(0.11, 0.14, 0.21);
const PRESS_COLOR: Color = Color::srgb(0.16, 0.21, 0.31);
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
const PLAYER: Color = Color::srgb(0.40, 0.78, 1.0);
const COMPUTER: Color = Color::srgb(0.99, 0.62, 0.35);
const DRAW: Color = Color::srgb(0.48, 0.54, 0.66);
const PANEL_COLOR: Color = Color::srgb(0.05, 0.06, 0.10);

#[derive(Component)]
pub(crate) struct Mark(usize);

#[derive(Component)]
pub(crate) struct Status;

#[derive(Component)]
pub(crate) struct Dashboard;

#[derive(Component)]
pub(crate) struct ProgressFill;

#[derive(Component)]
pub(crate) struct Readout(Reading);

#[derive(Clone, Copy)]
pub(crate) enum Reading {
    Count,
    Percent,
    Total(Ending),
    Latest,
    Average,
}

#[derive(Component)]
pub(crate) struct ChunkBar {
    chunk: usize,
    ending: Ending,
}

#[derive(Component)]
pub(crate) struct Recent(usize);

type PanelNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static Dashboard>,
        Option<&'static ProgressFill>,
        Option<&'static ChunkBar>,
        Option<&'static Recent>,
        &'static mut BackgroundColor,
    ),
    Or<(
        With<Dashboard>,
        With<ProgressFill>,
        With<ChunkBar>,
        With<Recent>,
    )>,
>;

type StatusText<'w, 's> =
    Query<'w, 's, (&'static mut Text, &'static mut TextColor), (With<Status>, Without<Mark>)>;

#[derive(Resource, Default)]
pub(crate) struct Refresh {
    elapsed: f32,
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(Refresh::default());
    commands
        .spawn((ui::screen(), Pickable::IGNORE))
        .with_children(|screen| {
            screen.spawn((Text::new(HEADING), ui::text(30.0, TITLE)));
            screen
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        height: Val::Auto,
                        column_gap: Val::Px(24.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(BOARD),
                            height: Val::Auto,
                            align_self: AlignSelf::Center,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|left| {
                        left.spawn((
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
                                                    width: Val::Px(TILE),
                                                    height: Val::Px(TILE),
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
                        left.spawn((Status, Text::new("showcase"), ui::text(17.0, DIM)));
                    });
                    row.spawn((
                        Dashboard,
                        Node {
                            width: Val::Px(PANEL),
                            height: Val::Px(300.0),
                            align_self: AlignSelf::Center,
                            padding: UiRect::all(Val::Px(12.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(PANEL_COLOR),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| {
                        panel.spawn((Text::new("self-play"), ui::text(13.0, DIM)));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                line.spawn((
                                    Readout(Reading::Count),
                                    Text::new("0 / 0"),
                                    ui::text(24.0, TITLE),
                                ));
                                line.spawn((
                                    Readout(Reading::Percent),
                                    Text::new("0%"),
                                    ui::text(16.0, DIM),
                                ));
                            });
                        panel.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(4.0),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(TRACK),
                            children![(
                                ProgressFill,
                                Node {
                                    width: Val::Px(0.0),
                                    height: Val::Percent(100.0),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(FILL),
                            )],
                        ));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                line.spawn((
                                    Readout(Reading::Total(Ending::XWins)),
                                    Text::new("X 0 0%"),
                                    ui::text(13.0, PLAYER),
                                ));
                                line.spawn((
                                    Readout(Reading::Total(Ending::OWins)),
                                    Text::new("O 0 0%"),
                                    ui::text(13.0, COMPUTER),
                                ));
                                line.spawn((
                                    Readout(Reading::Total(Ending::Draw)),
                                    Text::new("draw 0 0%"),
                                    ui::text(13.0, DRAW),
                                ));
                            });
                        panel.spawn((Text::new("results per 1,000 games"), ui::text(12.0, DIM)));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Px(CONTENT),
                                    height: Val::Px(CHART),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(1.0),
                                    align_items: AlignItems::Stretch,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|chart| {
                                let chunks = TOTAL_GAMES.div_ceil(CHUNK) as usize;
                                for chunk in 0..chunks {
                                    chart
                                        .spawn((
                                            Node {
                                                width: Val::Px(3.0),
                                                height: Val::Percent(100.0),
                                                flex_direction: FlexDirection::Column,
                                                justify_content: JustifyContent::FlexEnd,
                                                ..default()
                                            },
                                            BackgroundColor(TRACK),
                                        ))
                                        .with_children(|column| {
                                            for ending in
                                                [Ending::OWins, Ending::Draw, Ending::XWins]
                                            {
                                                column.spawn((
                                                    ChunkBar { chunk, ending },
                                                    Node {
                                                        width: Val::Percent(100.0),
                                                        height: Val::Px(0.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(shade(ending)),
                                                ));
                                            }
                                        });
                                }
                            });
                        panel
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                line.spawn((
                                    Text::new("recent 128 games | height = moves"),
                                    ui::text(12.0, DIM),
                                ));
                                line.spawn((
                                    Readout(Reading::Average),
                                    Text::new("avg 0.0 moves"),
                                    ui::text(12.0, DIM),
                                ));
                            });
                        panel.spawn((
                            Readout(Reading::Latest),
                            Text::new("latest: none"),
                            ui::text(12.0, DIM),
                        ));
                        panel
                            .spawn((
                                Node {
                                    width: Val::Px(CONTENT),
                                    height: Val::Px(RECENT_STRIP),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::FlexEnd,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|strip| {
                                for index in 0..RECENT {
                                    strip.spawn((
                                        Recent(index),
                                        Node {
                                            width: Val::Px(3.0),
                                            height: Val::Px(0.0),
                                            ..default()
                                        },
                                        BackgroundColor(TRACK),
                                    ));
                                }
                            });
                    });
                });
        });
}

pub fn paint_board(
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
    let (held, shade) = match game.stage() {
        Stage::Learning => {
            if game.trained() {
                ("training complete".to_string(), GOOD)
            } else {
                match game.showcase() {
                    Some(record) => (
                        format!("{} in {} moves", ending_words(record.ending), record.moves),
                        shade(record.ending),
                    ),
                    None => ("showcase".to_string(), DIM),
                }
            }
        }
        Stage::Done(Verdict::Win) => ("you win".to_string(), GOOD),
        Stage::Done(Verdict::Draw) => ("draw".to_string(), TITLE),
        Stage::Done(Verdict::Loss) => ("you lose".to_string(), BAD),
        Stage::Human => ("your move".to_string(), DIM),
        Stage::Computer => ("thinking".to_string(), DIM),
    };
    for (mut text, mut colour) in &mut status {
        if text.0 != held {
            text.0 = held.clone();
        }
        if colour.0 != shade {
            colour.0 = shade;
        }
    }
}

pub fn paint_dashboard(
    time: Res<Time>,
    mut refresh: ResMut<Refresh>,
    game: Res<Game>,
    mut panels: PanelNodes,
    mut readouts: Query<(&Readout, &mut Text, &mut TextColor)>,
) {
    let learning = game.stage() == Stage::Learning;
    for (mut node, dashboard, _, _, _, _) in &mut panels {
        if dashboard.is_some() {
            let display = if learning {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }
    }
    if !learning {
        return;
    }
    refresh.elapsed += time.delta_secs();
    if refresh.elapsed < REFRESH {
        return;
    }
    refresh.elapsed = 0.0;
    let results = game.results();
    for (readout, mut text, mut colour) in &mut readouts {
        let (held, shade) = match readout.0 {
            Reading::Count => (
                format!(
                    "{} / {}",
                    grouped(results.completed()),
                    grouped(results.total())
                ),
                TITLE,
            ),
            Reading::Percent => (
                format!("{}%", (game.progress() * 100.0).round() as u32),
                DIM,
            ),
            Reading::Total(ending) => {
                let count = count(results, ending);
                (
                    format!(
                        "{} {} {}%",
                        ending_text(ending),
                        grouped(count),
                        share(count, results.completed())
                    ),
                    shade(ending),
                )
            }
            Reading::Latest => match results.recent().back() {
                Some(record) => (
                    format!(
                        "latest: {}, {} moves",
                        ending_words(record.ending),
                        record.moves
                    ),
                    shade(record.ending),
                ),
                None => ("latest: none".to_string(), DIM),
            },
            Reading::Average => (format!("avg {:.1} moves", results.average_moves()), DIM),
        };
        if text.0 != held {
            text.0 = held;
        }
        if colour.0 != shade {
            colour.0 = shade;
        }
    }
    for (mut node, _, fill, chunk, recent, mut colour) in &mut panels {
        if fill.is_some() {
            node.width = Val::Px(CONTENT * game.progress());
            continue;
        }
        if let Some(bar) = chunk {
            let count = results.chunks()[bar.chunk].count(bar.ending);
            node.height = Val::Percent(count as f32 / CHUNK as f32 * 100.0);
            continue;
        }
        if let Some(tile) = recent {
            match results.recent().get(tile.0) {
                Some(record) => {
                    node.height = Val::Px(6.0 + f32::from((record.moves - 5) as u8) * 3.0);
                    colour.0 = shade(record.ending);
                }
                None => {
                    node.height = Val::Px(0.0);
                    colour.0 = TRACK;
                }
            }
        }
    }
}

fn count(results: &Results, ending: Ending) -> u32 {
    match ending {
        Ending::XWins => results.x_wins(),
        Ending::OWins => results.o_wins(),
        Ending::Draw => results.draws(),
    }
}

fn share(count: u32, total: u32) -> u32 {
    if total == 0 {
        return 0;
    }
    (u64::from(count) * 100 / u64::from(total)) as u32
}

fn ending_text(ending: Ending) -> &'static str {
    match ending {
        Ending::XWins => "X",
        Ending::OWins => "O",
        Ending::Draw => "draw",
    }
}

fn ending_words(ending: Ending) -> &'static str {
    match ending {
        Ending::XWins => "X wins",
        Ending::OWins => "O wins",
        Ending::Draw => "draw",
    }
}

fn shade(ending: Ending) -> Color {
    match ending {
        Ending::XWins => PLAYER,
        Ending::OWins => COMPUTER,
        Ending::Draw => DRAW,
    }
}

fn grouped(value: u32) -> String {
    let digits = value.to_string();
    let mut held = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            held.push(',');
        }
        held.push(digit);
    }
    held
}
