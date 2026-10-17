use crate::display::Display;
use crate::env::{Action, Ending};
use crate::policy::ACTIONS;
use crate::train::Training;
use bevy::prelude::*;
use bevy_neura_example::probability::softmax;
use bevy_neura_example::theme::{BAD, DIM, GOOD};
use bevy_neura_example::ui::{at, bottom, right, text};

const INK: Color = Color::srgb(0.78, 0.82, 0.94);
const HOT: Color = Color::srgb(0.98, 0.7, 0.32);
const PANEL: Color = Color::srgba(0.05, 0.06, 0.12, 0.72);
const BARS: usize = 96;
const BAR_STEP: f32 = 2.4;
const CHART_LEFT: f32 = 20.0;
const CHART_BOTTOM: f32 = 92.0;
const CHART_HEIGHT: f32 = 96.0;
const CHART_FLOOR: f32 = -150.0;
const CHART_CEILING: f32 = 350.0;

#[derive(Component)]
pub(crate) enum Line {
    Training,
    Showcase,
    Banner,
}

#[derive(Component)]
pub(crate) struct Bar(usize);

#[derive(Component)]
pub(crate) struct Meter(usize);

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(CHART_LEFT - 4.0),
            bottom: Val::Px(CHART_BOTTOM - 4.0),
            width: Val::Px(BARS as f32 * BAR_STEP + 8.0),
            height: Val::Px(CHART_HEIGHT + 8.0),
            ..default()
        },
        BackgroundColor(PANEL),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(CHART_LEFT),
            bottom: Val::Px(CHART_BOTTOM + share(0.0) * CHART_HEIGHT),
            width: Val::Px(BARS as f32 * BAR_STEP),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(DIM.with_alpha(0.7)),
    ));
    for index in 0..BARS {
        commands.spawn((
            Bar(index),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CHART_LEFT + index as f32 * BAR_STEP),
                width: Val::Px(BAR_STEP - 0.6),
                bottom: Val::Px(CHART_BOTTOM),
                height: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(BAD),
        ));
    }
    commands.spawn((
        Text::new(format!("reward of the last {BARS} episodes")),
        text(12.0, DIM),
        bottom(CHART_LEFT, CHART_BOTTOM + CHART_HEIGHT + 8.0),
    ));
    commands.spawn((
        Line::Training,
        Text::new(""),
        text(15.0, INK),
        at(16.0, 12.0),
    ));
    commands.spawn((
        Line::Showcase,
        Text::new(""),
        text(15.0, INK),
        right(16.0, 12.0),
    ));
    commands.spawn((
        Text::new("policy of the showcase"),
        text(13.0, DIM),
        right(16.0, 214.0),
    ));
    for action in 0..ACTIONS {
        commands.spawn((
            Meter(action),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(150.0),
                top: Val::Px(238.0 + action as f32 * 17.0),
                width: Val::Px(0.0),
                height: Val::Px(11.0),
                ..default()
            },
            BackgroundColor(HOT.with_alpha(0.85)),
        ));
        commands.spawn((
            Text::new(Action::ALL[action].name()),
            text(12.0, DIM),
            right(103.0, 237.0 + action as f32 * 17.0),
        ));
    }
    commands.spawn((
        Text::new("space turbo   p pause   r replay"),
        text(15.0, DIM),
        bottom(16.0, 16.0),
    ));
    commands.spawn((
        Line::Banner,
        Text::new(""),
        TextLayout::justify(Justify::Center),
        text(26.0, INK),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(300.0),
            ..default()
        },
    ));
}

type Lines<'w, 's> = Query<'w, 's, (&'static Line, &'static mut Text, &'static mut TextColor)>;
type Bars<'w, 's> = Query<
    'w,
    's,
    (
        &'static Bar,
        &'static mut Node,
        &'static mut BackgroundColor,
    ),
>;
type Meters<'w, 's> = Query<'w, 's, (&'static Meter, &'static mut Node), Without<Bar>>;

pub fn paint(
    training: Res<Training>,
    display: Res<Display>,
    mut lines: Lines,
    mut bars: Bars,
    mut meters: Meters,
) {
    for (line, mut text, mut color) in &mut lines {
        let (drawn, ink) = match line {
            Line::Training => (
                format!(
                    "training {}   {} episodes   mean {:+.1}",
                    speed(training.steps),
                    training.episodes,
                    training.mean(),
                ),
                INK,
            ),
            Line::Showcase => (
                format!(
                    "showcase   landed {} of {}",
                    display.landed, display.episodes,
                ),
                INK,
            ),
            Line::Banner => (banner(&display), banner_ink(&display)),
        };
        text.0 = drawn;
        color.0 = ink;
    }
    let probabilities = softmax(&display.logits);
    for (meter, mut node) in &mut meters {
        node.width = Val::Px(6.0 + 50.0 * probabilities[meter.0]);
    }
    let history = &training.history;
    for (bar, mut node, mut background) in &mut bars {
        let reward = history
            .get(history.len().saturating_sub(BARS) + bar.0)
            .copied();
        let Some(reward) = reward else {
            node.height = Val::Px(0.0);
            continue;
        };
        let zero = share(0.0);
        let at = share(reward);
        node.bottom = Val::Px(CHART_BOTTOM + zero.min(at) * CHART_HEIGHT);
        node.height = Val::Px((at - zero).abs() * CHART_HEIGHT);
        background.0 = if reward >= 0.0 { GOOD } else { BAD };
    }
}

fn speed(steps: u32) -> String {
    match steps {
        0 => "paused".to_string(),
        steps => format!("x{steps}"),
    }
}

fn banner(display: &Display) -> String {
    match display.ending {
        Some(Ending::Landed) => "landed".to_string(),
        Some(Ending::Crashed) => "crashed".to_string(),
        Some(Ending::Departed) => "lost in space".to_string(),
        None => String::new(),
    }
}

fn banner_ink(display: &Display) -> Color {
    match display.ending {
        Some(Ending::Landed) => GOOD,
        Some(_) => BAD,
        None => INK,
    }
}

fn share(reward: f32) -> f32 {
    ((reward - CHART_FLOOR) / (CHART_CEILING - CHART_FLOOR)).clamp(0.0, 1.0)
}
