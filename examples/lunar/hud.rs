use crate::display::Display;
use crate::env::{Action, Ending};
use crate::policy::ACTIONS;
use crate::train::Training;
use bevy::prelude::*;
use bevy_neura::NeuraRuntime;

const INK: Color = Color::srgb(0.78, 0.82, 0.94);
const DIM: Color = Color::srgb(0.55, 0.6, 0.76);
const HOT: Color = Color::srgb(0.98, 0.7, 0.32);
const GOOD: Color = Color::srgb(0.45, 0.86, 0.55);
const BAD: Color = Color::srgb(0.9, 0.42, 0.42);
const PANEL: Color = Color::srgba(0.05, 0.06, 0.12, 0.72);
const BARS: usize = 96;
const BAR_STEP: f32 = 2.4;
const CHART_LEFT: f32 = 20.0;
const CHART_BOTTOM: f32 = 92.0;
const CHART_HEIGHT: f32 = 96.0;
const CHART_FLOOR: f32 = -150.0;
const CHART_CEILING: f32 = 250.0;
const SOLVED: f32 = 200.0;
const FLASH: u32 = 30;

#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) enum Line {
    Title,
    Device,
    Training,
    Showcase,
    Losses,
    Hints,
    Banner,
    Policy,
}

#[derive(Component)]
pub(crate) struct Bar(usize);

#[derive(Component)]
pub(crate) struct Policy(u8);

#[derive(Component)]
pub(crate) struct Flash;

#[derive(Resource)]
pub struct Hud {
    device: String,
    flashed: u32,
}

pub fn setup(mut commands: Commands, runtime: Res<NeuraRuntime>) {
    commands.spawn((
        Flash,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
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
    for (reward, color) in [(SOLVED, GOOD.with_alpha(0.45)), (0.0, DIM.with_alpha(0.7))] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CHART_LEFT),
                bottom: Val::Px(CHART_BOTTOM + share(reward) * CHART_HEIGHT),
                width: Val::Px(BARS as f32 * BAR_STEP),
                height: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(color),
        ));
    }
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
        Line::Policy,
        Text::new("policy"),
        text(13.0, DIM),
        right(16.0, 214.0),
    ));
    for action in 0..ACTIONS {
        commands.spawn((
            Policy(action as u8),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(150.0),
                top: Val::Px(238.0 + action as f32 * 17.0),
                width: Val::Px(56.0),
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
        Line::Title,
        Text::new("lunar lander - PPO on bevy-neura"),
        text(20.0, INK),
        at(16.0, 12.0),
    ));
    commands.spawn((Line::Device, Text::new(""), text(13.0, DIM), at(16.0, 36.0)));
    commands.spawn((
        Line::Training,
        Text::new(""),
        text(15.0, INK),
        at(16.0, 56.0),
    ));
    commands.spawn((Line::Losses, Text::new(""), text(15.0, INK), at(16.0, 78.0)));
    commands.spawn((
        Line::Showcase,
        Text::new(""),
        text(15.0, INK),
        right(16.0, 12.0),
    ));
    commands.spawn((
        Line::Hints,
        Text::new("space turbo   h pilot   p pause   r replay"),
        text(15.0, DIM),
        bottom(16.0),
    ));
    commands.spawn((
        Line::Banner,
        Text::new(""),
        TextLayout::justify(Justify::Center),
        text(32.0, INK),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(300.0),
            ..default()
        },
    ));
    commands.insert_resource(Hud {
        device: runtime.context().adapter_info().name.clone(),
        flashed: 0,
    });
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
    (Without<Policy>, Without<Flash>),
>;
type PolicyBars<'w, 's> =
    Query<'w, 's, (&'static Policy, &'static mut Node), (Without<Bar>, Without<Flash>)>;
type FlashColor<'w, 's> = Query<
    'w,
    's,
    (&'static mut BackgroundColor, &'static mut Visibility),
    (With<Flash>, Without<Bar>, Without<Policy>),
>;

pub fn paint(
    training: Res<Training>,
    display: Res<Display>,
    mut hud: ResMut<Hud>,
    mut lines: Lines,
    mut bars: Bars,
    mut policy: PolicyBars,
    mut flash: FlashColor,
) {
    let device = hud.device.clone();
    for (line, mut text, mut color) in &mut lines {
        let (drawn, ink) = match line {
            Line::Title => ("lunar lander - PPO on bevy-neura".to_string(), INK),
            Line::Device => (format!("[{}]", device), DIM),
            Line::Policy => ("policy of the showcase".to_string(), DIM),
            Line::Training => (
                format!(
                    "learn   {} samples   {} updates   {:.0}/s   {} episodes   mean {:+.1}   best {:+.1}",
                    samples(training.samples),
                    training.updates,
                    training.steps_per_second,
                    training.episodes,
                    training.mean(),
                    training.best,
                ),
                INK,
            ),
            Line::Showcase => (
                format!(
                    "showcase ({})\nepisode {:+.1}   value {:+.1}\nmean {:+.1}   best {:+.1}   landed {} times",
                    display.pilot.name(),
                    display.returned,
                    display.value,
                    display.mean(),
                    if display.best == f32::MIN {
                        0.0
                    } else {
                        display.best
                    },
                    display.episodes,
                ),
                INK,
            ),
            Line::Losses => (
                format!(
                    "loss {:>7.2}   policy {:>+6.3}   value {:>8.2}   entropy {:.3}   explained {:.2}",
                    training.diagnostics.loss,
                    training.diagnostics.policy,
                    training.diagnostics.value,
                    training.diagnostics.entropy,
                    training.explained,
                ),
                INK,
            ),
            Line::Hints => (
                "space turbo   h pilot   p pause   r replay".to_string(),
                DIM,
            ),
            Line::Banner => (banner(&display), banner_ink(&display)),
        };
        text.0 = drawn;
        color.0 = ink;
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
        background.0 = if reward >= SOLVED {
            GOOD
        } else if reward >= 0.0 {
            GOOD.with_alpha(0.9)
        } else {
            BAD.with_alpha(0.95)
        };
    }
    let probabilities = softmax(&display.logits);
    for (bar, mut node) in &mut policy {
        node.width = Val::Px(6.0 + 50.0 * probabilities[bar.0 as usize]);
    }
    let (mut color, mut place) = flash.single_mut().expect("the screen shows one flash");
    if display.fresh() && hud.flashed == 0 {
        hud.flashed = FLASH;
    }
    if hud.flashed == 0 {
        *place = Visibility::Hidden;
        color.0 = Color::NONE;
        return;
    }
    hud.flashed -= 1;
    let fade = hud.flashed as f32 / FLASH as f32;
    *place = Visibility::Inherited;
    color.0 = match display.ending {
        Some(Ending::Landed) => Color::srgba(0.4, 1.0, 0.5, 0.2 * fade),
        Some(_) => Color::srgba(1.0, 0.35, 0.35, 0.24 * fade),
        None => Color::NONE,
    };
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let highest = logits.iter().copied().fold(f32::MIN, f32::max);
    let weights = logits
        .iter()
        .map(|logit| (logit - highest).exp())
        .collect::<Vec<f32>>();
    let total = weights.iter().sum::<f32>();
    weights.into_iter().map(|weight| weight / total).collect()
}

fn banner(display: &Display) -> String {
    match display.ending {
        Some(Ending::Landed) => "landed   +100".to_string(),
        Some(Ending::Crashed) => "crashed   -100".to_string(),
        Some(Ending::Departed) => "lost in space   -100".to_string(),
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

fn samples(count: u64) -> String {
    match count {
        0..=9_999 => count.to_string(),
        10_000..=999_999 => format!("{:.0}k", count as f64 / 1e3),
        _ => format!("{:.2}M", count as f64 / 1e6),
    }
}

fn text(size: f32, color: Color) -> (TextFont, TextColor) {
    (
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn at(left: f32, top: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        ..default()
    }
}

fn right(right: f32, top: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        right: Val::Px(right),
        top: Val::Px(top),
        ..default()
    }
}

fn bottom(bottom: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(CHART_LEFT),
        bottom: Val::Px(bottom),
        ..default()
    }
}
