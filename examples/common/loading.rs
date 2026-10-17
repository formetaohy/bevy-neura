use crate::theme::{DIM, TITLE};
use crate::transfer::{Report, Transfer};
use crate::ui;
use bevy::picking::Pickable;
use bevy::prelude::*;

const WIDTH: f32 = 360.0;
const TRACK_HEIGHT: f32 = 8.0;
const TRACK: Color = Color::srgb(0.09, 0.12, 0.19);
const FILL: Color = Color::srgb(0.36, 0.86, 0.78);
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

    pub fn follow(&mut self, transfer: &mut Option<Transfer>) {
        let report = transfer.as_ref().map(Transfer::report);
        match report {
            None => self.report(1.0),
            Some(Report::Done) => {
                *transfer = None;
                self.report(1.0);
            }
            Some(Report::Failed(error)) => panic!("the artifacts do not download: {error}"),
            Some(Report::Measuring) => self.report(0.0),
            Some(Report::Fetching { done, total }) if total > 0 => {
                self.report(done as f32 / total as f32);
            }
            Some(Report::Fetching { .. }) => self.report(0.0),
        }
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
        ui::screen(),
        Pickable::IGNORE,
        children![
            (Text::new(title), ui::text(34.0, TITLE)),
            (Mark(Part::Label), Text::default(), ui::text(18.0, DIM)),
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
            (Mark(Part::Percent), Text::new("0%"), ui::text(18.0, DIM)),
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
