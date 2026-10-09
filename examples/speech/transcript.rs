use crate::speech::{Speech, Stage};
use crate::utterance::Utterance;
use bevy::picking::Pickable;
use bevy::prelude::*;
use std::collections::VecDeque;

const SLOTS: usize = 6;
const SMALL: f32 = 17.0;
const LARGE: f32 = 33.0;
const DIM: Color = Color::srgb(0.55, 0.62, 0.76);

#[derive(Component)]
pub(crate) struct Line(usize);

#[derive(Component)]
pub(crate) struct Status;

#[derive(Resource, Default)]
pub(crate) struct History(VecDeque<String>);

pub fn setup(mut commands: Commands) {
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
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(12.0),
                padding: UiRect {
                    left: Val::Px(44.0),
                    right: Val::Px(44.0),
                    top: Val::Px(60.0),
                    bottom: Val::Px(268.0),
                },
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|screen| {
            for slot in 0..SLOTS {
                screen.spawn((
                    Line(slot),
                    Text::default(),
                    TextFont::from_font_size(size(slot)),
                    TextColor(shade(slot)),
                    TextLayout::justify(Justify::Center),
                ));
            }
            screen.spawn((
                Status,
                Text::new(prompt(Stage::Idle)),
                TextFont::from_font_size(SMALL),
                TextColor(DIM),
                TextLayout::justify(Justify::Center),
            ));
        });
    commands.init_resource::<History>();
}

pub fn write(
    mut utterances: MessageReader<Utterance>,
    mut history: ResMut<History>,
    mut lines: Query<(&Line, &mut Text)>,
) {
    let mut landed = false;
    for utterance in utterances.read() {
        let text = utterance.text.trim();
        if text.is_empty() {
            continue;
        }
        if history.0.len() == SLOTS {
            history.0.pop_front();
        }
        history.0.push_back(text.to_string());
        landed = true;
    }
    if !landed {
        return;
    }
    let count = history.0.len() as isize;
    for (line, mut text) in &mut lines {
        let index = count + line.0 as isize - SLOTS as isize;
        let words = if index < 0 {
            String::new()
        } else {
            history.0[index as usize].clone()
        };
        if text.0 != words {
            text.0 = words;
        }
    }
}

pub fn paint(speech: Res<Speech>, mut statuses: Query<&mut Text, With<Status>>) {
    let words = prompt(speech.stage());
    for mut text in &mut statuses {
        if text.0 != words {
            text.0 = words.to_string();
        }
    }
}

fn prompt(stage: Stage) -> &'static str {
    match stage {
        Stage::Idle => "hold the button and speak",
        Stage::Recording => "listening",
        Stage::Reading => "reading the sound",
        Stage::Writing => "writing the words",
    }
}

fn size(slot: usize) -> f32 {
    SMALL + (LARGE - SMALL) * rank(slot)
}

fn shade(slot: usize) -> Color {
    let rank = rank(slot);
    Color::srgba(0.93, 0.96, 1.0, 0.20 + 0.80 * rank * rank)
}

fn rank(slot: usize) -> f32 {
    slot as f32 / (SLOTS - 1) as f32
}
