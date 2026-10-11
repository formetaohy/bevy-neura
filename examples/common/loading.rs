use bevy::picking::Pickable;
use bevy::prelude::*;

const WIDTH: f32 = 360.0;
const TRACK_HEIGHT: f32 = 8.0;
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
const TITLE: Color = Color::srgb(0.94, 0.97, 1.0);
const DIM: Color = Color::srgb(0.55, 0.62, 0.76);
const SHOWN_AFTER: u32 = 3;

#[derive(Resource)]
pub struct Loading {
    steps: Vec<Step>,
    index: usize,
    part: f32,
    frames: u32,
}

struct Step {
    label: &'static str,
    share: f32,
}

impl Loading {
    pub fn of(steps: &[(&'static str, f32)]) -> Self {
        assert!(!steps.is_empty(), "a loading page walks at least one step");
        let shares = steps.iter().map(|(_, share)| *share).sum::<f32>();
        assert!(
            (shares - 1.0).abs() < 1e-3,
            "the steps of a loading page share {shares} of its work",
        );
        Self {
            steps: steps
                .iter()
                .map(|(label, share)| Step {
                    label,
                    share: *share,
                })
                .collect(),
            index: 0,
            part: 0.0,
            frames: 0,
        }
    }

    pub fn shown(&mut self) -> bool {
        self.frames = self.frames.saturating_add(1);
        self.frames >= SHOWN_AFTER
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn label(&self) -> &'static str {
        self.steps[self.index.min(self.steps.len() - 1)].label
    }

    pub fn progress(&self) -> f32 {
        let done = self.steps[..self.index]
            .iter()
            .map(|step| step.share)
            .sum::<f32>();
        let part = self
            .steps
            .get(self.index)
            .map_or(0.0, |step| step.share * self.part);
        (done + part).clamp(0.0, 1.0)
    }

    pub fn report(&mut self, part: f32) {
        assert!(
            self.index < self.steps.len(),
            "a loading page of {} steps reports no more work",
            self.steps.len(),
        );
        let part = part.clamp(0.0, 1.0);
        if part < 1.0 {
            self.part = part;
            return;
        }
        self.index += 1;
        self.part = 0.0;
    }
}

pub struct LoadingPlugin<S: States> {
    state: S,
    title: &'static str,
}

impl<S: States> LoadingPlugin<S> {
    pub fn new(state: S, title: &'static str) -> Self {
        Self { state, title }
    }
}

impl<S: States> Plugin for LoadingPlugin<S> {
    fn build(&self, app: &mut App) {
        let title = self.title;
        app.add_systems(
            OnEnter(self.state.clone()),
            move |mut commands: Commands| {
                screen(&mut commands, title);
            },
        )
        .add_systems(OnExit(self.state.clone()), clear)
        .add_systems(Update, paint.run_if(in_state(self.state.clone())));
    }
}

#[derive(Component)]
struct Screen;

#[derive(Component)]
struct Fill;

#[derive(Component)]
struct Mark(Part);

enum Part {
    Label,
    Percent,
}

fn screen(commands: &mut Commands, title: &str) {
    commands.spawn((
        Screen,
        Pickable::IGNORE,
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
        children![
            (
                Text::new(title),
                TextFont::from_font_size(34.0),
                TextColor(TITLE),
            ),
            (
                Mark(Part::Label),
                Text::default(),
                TextFont::from_font_size(18.0),
                TextColor(DIM),
            ),
            (
                Node {
                    width: Val::Px(WIDTH),
                    height: Val::Px(TRACK_HEIGHT),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(TRACK),
                children![(
                    Fill,
                    Node {
                        width: Val::Px(0.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(FILL),
                )],
            ),
            (
                Mark(Part::Percent),
                Text::new("0%"),
                TextFont::from_font_size(18.0),
                TextColor(DIM),
            ),
        ],
    ));
}

fn clear(mut commands: Commands, screens: Query<Entity, With<Screen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Loading>();
}

fn paint(
    loading: Res<Loading>,
    mut fills: Query<&mut Node, With<Fill>>,
    mut marks: Query<(&Mark, &mut Text)>,
) {
    let progress = loading.progress();
    for mut node in &mut fills {
        node.width = Val::Px(WIDTH * progress);
    }
    let percent = format!("{}%", (progress * 100.0).round() as u32);
    for (mark, mut text) in &mut marks {
        match mark.0 {
            Part::Label => {
                if text.0.as_str() != loading.label() {
                    text.0 = loading.label().to_string();
                }
            }
            Part::Percent => {
                if text.0 != percent {
                    text.0 = percent.clone();
                }
            }
        }
    }
}
