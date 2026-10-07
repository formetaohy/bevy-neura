use crate::game::Game;
use crate::speech::{PushToTalk, Speech};
use bevy::prelude::*;
use bevy::text::FontSize;

const IDLE: Color = Color::srgb(0.30, 0.72, 0.95);
const LISTENING: Color = Color::srgb(0.95, 0.35, 0.40);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Readout {
    Head,
    Heard,
    Help,
    Over,
}

#[derive(Component)]
pub(crate) struct Talk;

#[derive(Component)]
pub(crate) struct Overlay;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        children![
            (
                Text::new("loading"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.90, 1.0)),
                Readout::Head,
            ),
            (
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.70)),
                Readout::Heard,
            ),
            (
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.68, 0.80)),
                Readout::Help,
            ),
        ],
    ));
    let button = commands
        .spawn((
            Button,
            Talk,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(26.0),
                left: Val::Px(330.0),
                width: Val::Px(300.0),
                height: Val::Px(62.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(IDLE),
            children![(
                Text::new("按住说话 (空格)"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.05, 0.07, 0.10)),
            )],
        ))
        .id();
    commands.entity(button).observe(pressed);
    commands.entity(button).observe(released);
    commands.spawn((
        Overlay,
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            left: Val::Px(0.0),
            top: Val::Px(220.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Text::new("GAME OVER — 说 restart 或按 R 重来"),
            TextFont {
                font_size: FontSize::Px(40.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.45, 0.45)),
            Readout::Over,
        )],
    ));
}

fn pressed(_press: On<Pointer<Press>>, mut push: ResMut<PushToTalk>) {
    push.pressing = true;
}

fn released(_release: On<Pointer<Release>>, mut push: ResMut<PushToTalk>) {
    push.pressing = false;
}

pub fn talk(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    speech: Option<Res<Speech>>,
    mut push: ResMut<PushToTalk>,
    mut talks: Query<&mut BackgroundColor, With<Talk>>,
) {
    if push.pressing && !mouse.pressed(MouseButton::Left) {
        push.pressing = false;
    }
    push.held = push.pressing || keys.pressed(KeyCode::Space);
    let listening = speech.map(|speech| speech.busy()).unwrap_or(false);
    for mut color in &mut talks {
        *color = BackgroundColor(if listening { LISTENING } else { IDLE });
    }
}

pub fn restart(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    if keys.just_pressed(KeyCode::KeyR) {
        *game = Game::default();
    }
}

pub fn update(
    speech: Option<Res<Speech>>,
    game: Res<Game>,
    mut heard: MessageReader<crate::speech::Transcription>,
    mut last: Local<String>,
    mut readouts: Query<(&mut Text, &Readout)>,
    mut overlays: Query<&mut Node, With<Overlay>>,
) {
    for transcription in heard.read() {
        *last = transcription.text.clone();
    }
    for mut node in &mut overlays {
        node.display = if game.over {
            Display::Flex
        } else {
            Display::None
        };
    }
    let head = match speech.as_ref() {
        None => "loading a local whisper model…".to_string(),
        Some(speech) => format!(
            "score {}   life {}   language {}   speech {:.1} s   [{}]",
            game.score,
            game.health.max(0),
            speech.language(),
            speech.seconds(),
            speech.state(),
        ),
    };
    let heard = last.clone();
    let help = format!(
        "heard -> {}   ·   voice: left right up down fire freeze restart   ·   keys: wasd, space to talk, r to reset",
        game.command.name(),
    );
    for (mut text, readout) in &mut readouts {
        match readout {
            Readout::Head => text.0 = head.clone(),
            Readout::Heard => text.0 = heard.clone(),
            Readout::Help => text.0 = help.clone(),
            Readout::Over => {}
        }
    }
}
