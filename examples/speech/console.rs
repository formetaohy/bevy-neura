use crate::speech::{Record, Speech, Stage};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;

const SIZE: f32 = 132.0;
const CORE: f32 = 46.0;
const LOUD: f32 = 0.09;
const IDLE: Color = Color::srgb(0.13, 0.16, 0.23);
const IDLE_CORE: Color = Color::srgb(0.44, 0.52, 0.68);
const HELD: Color = Color::srgb(0.32, 0.10, 0.13);
const HELD_CORE: Color = Color::srgb(0.99, 0.32, 0.36);
const BUSY: Color = Color::srgb(0.10, 0.12, 0.18);
const BUSY_CORE: Color = Color::srgb(0.99, 0.72, 0.32);

#[derive(Component)]
pub(crate) struct Core;

type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut BackgroundColor,
        Has<Pressed>,
        Has<InteractionDisabled>,
    ),
    With<Record>,
>;

type Cores<'w, 's> =
    Query<'w, 's, (&'static mut Node, &'static mut BackgroundColor), (With<Core>, Without<Record>)>;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(40.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Button,
            Record,
            Node {
                width: Val::Px(SIZE),
                height: Val::Px(SIZE),
                border_radius: BorderRadius::MAX,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(IDLE),
            children![(
                Core,
                Node {
                    width: Val::Px(CORE),
                    height: Val::Px(CORE),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(IDLE_CORE),
            )],
        )],
    ));
}

pub fn paint(
    time: Res<Time>,
    speech: Res<Speech>,
    mut commands: Commands,
    mut buttons: Buttons,
    mut cores: Cores,
) {
    let stage = speech.stage();
    let level = (speech.level() / LOUD).clamp(0.0, 1.0).sqrt();
    let beat = 0.5 + 0.5 * (time.elapsed_secs() * 5.0).sin();
    let busy = matches!(stage, Stage::Reading | Stage::Writing);
    for (entity, mut background, pressed, disabled) in &mut buttons {
        background.0 = if busy {
            BUSY
        } else if pressed {
            HELD
        } else {
            IDLE
        };
        match (busy, disabled) {
            (true, false) => {
                commands
                    .entity(entity)
                    .insert(InteractionDisabled)
                    .remove::<Pressed>();
            }
            (false, true) => {
                commands.entity(entity).remove::<InteractionDisabled>();
            }
            _ => {}
        }
    }
    for (mut node, mut background) in &mut cores {
        let reach = match stage {
            Stage::Idle => 0.0,
            Stage::Recording => level,
            Stage::Reading | Stage::Writing => beat,
        };
        let size = CORE + (SIZE - 44.0 - CORE) * reach;
        node.width = Val::Px(size);
        node.height = Val::Px(size);
        background.0 = match stage {
            Stage::Idle => IDLE_CORE,
            Stage::Recording => HELD_CORE,
            Stage::Reading | Stage::Writing => BUSY_CORE,
        };
    }
}
