use crate::flap::Flap;
use crate::game::{Game, Stage};
use crate::speech::Speech;
use crate::utterance::Utterance;
use bevy::prelude::*;

const HUD_COLOR: Color = Color::srgb(0.72, 0.78, 0.92);
const HUD_HOT: Color = Color::srgb(0.95, 0.45, 0.45);

#[derive(Component, Clone, Copy, PartialEq)]
pub enum Hud {
    Score,
    Voice,
    Notice,
    Hint,
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Hud::Score,
        Text::new(""),
        font(20.0),
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(12.0),
            ..default()
        },
    ));
    commands.spawn((
        Hud::Voice,
        Text::new(""),
        font(20.0),
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(40.0),
            ..default()
        },
    ));
    commands.spawn((
        Hud::Hint,
        Text::new(""),
        font(20.0),
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(14.0),
            ..default()
        },
    ));
    commands.spawn((
        Hud::Notice,
        Text::new(""),
        font(24.0),
        TextColor(HUD_COLOR),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Px(210.0),
            ..default()
        },
    ));
}

fn font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}

pub fn paint(
    game: Res<Game>,
    speech: Res<Speech>,
    mut utterances: MessageReader<Utterance>,
    mut last: Local<String>,
    mut huds: Query<(&mut Text, &mut TextColor, &Hud)>,
) {
    for utterance in utterances.read() {
        *last = utterance.text.clone();
    }
    let voice = if last.is_empty() {
        "heard nothing yet".to_string()
    } else {
        format!(
            "heard \"{}\" -> {}",
            last.as_str(),
            if Flap::read(last.as_str()).is_some() {
                "flap"
            } else {
                "nothing"
            },
        )
    };
    for (mut text, mut color, hud) in &mut huds {
        text.0 = match hud {
            Hud::Score => format!("score {}   best {}", game.score(), game.best()),
            Hud::Voice => format!("[{}] {voice}", speech.state()),
            Hud::Notice => notice(game.stage()).to_string(),
            Hud::Hint => "say fly to flap".to_string(),
        };
        color.0 = match hud {
            Hud::Voice if speech.busy() => HUD_HOT,
            _ => HUD_COLOR,
        };
    }
}

fn notice(stage: Stage) -> &'static str {
    match stage {
        Stage::Ready => "say fly to take off",
        Stage::Play | Stage::Falling => "",
        Stage::Over => "game over - say fly to fly again",
    }
}
