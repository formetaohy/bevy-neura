use bevy::prelude::*;
use std::time::Duration;

#[path = "../examples/speech/bird.rs"]
mod bird;
#[path = "../examples/speech/flap.rs"]
mod flap;
#[path = "../examples/speech/game.rs"]
mod game;
#[path = "../examples/speech/pipe.rs"]
mod pipe;
#[path = "../examples/speech/utterance.rs"]
mod utterance;
#[path = "../examples/speech/world.rs"]
mod world;

use bird::Bird;
use flap::Flap;
use game::{Game, Stage};
use pipe::Pipe;
use utterance::Utterance;

const STEP: f32 = 0.05;
const AIM: f32 = 30.0;

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Game>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<ColorMaterial>::default());
    app.insert_resource(Time::<()>::default());
    app.add_message::<Utterance>();
    app.add_systems(Startup, (world::setup, bird::setup));
    app.add_systems(Update, (game::direct, game::advance).chain());
    app.update();
    app
}

fn tick(app: &mut App, count: u32) {
    for _ in 0..count {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(STEP));
        app.update();
    }
}

fn say(app: &mut App, text: &str) {
    app.world_mut().write_message(Utterance {
        text: text.to_string(),
    });
}

fn bird_at(app: &mut App) -> Vec2 {
    let mut query = app.world_mut().query::<(&Bird, &Transform)>();
    query
        .single(app.world())
        .expect("a game holds one bird")
        .1
        .translation
        .xy()
}

fn next_gap(app: &mut App) -> f32 {
    let tail = bird_at(app).x - bird::RADIUS;
    let mut query = app.world_mut().query::<(&Pipe, &Transform)>();
    query
        .iter(app.world())
        .map(|(pipe, frame)| (frame.translation.x, pipe.gap))
        .filter(|(at, _)| at + pipe::WIDTH / 2.0 > tail)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map_or(bird::START_Y, |(_, gap)| gap)
}

fn flap_through(app: &mut App, ticks: u32) {
    for _ in 0..ticks {
        if bird_at(app).y < next_gap(app) - AIM {
            say(app, "fly");
        }
        tick(app, 1);
    }
}

#[test]
fn a_transcript_names_the_word_that_flaps() {
    for text in [
        "fly",
        " Fly!",
        "let me fly",
        "FLY.",
        "飞",
        "飛",
        "飞一下",
        "我想飞",
    ] {
        assert_eq!(Flap::read(text), Some(Flap), "{text:?} asks for no flap");
    }
    for text in [
        "",
        "  ",
        "butterfly",
        "flying",
        "sizzling",
        "(wind blowing)",
        "[BLANK_AUDIO]",
        "起来",
    ] {
        assert_eq!(Flap::read(text), None, "{text:?} asks for a flap");
    }
}

#[test]
fn a_spoken_word_starts_the_run_and_lifts_the_bird() {
    let mut app = app();
    tick(&mut app, 5);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Ready);
    let waiting = bird_at(&mut app).y;
    say(&mut app, " Fly!");
    tick(&mut app, 6);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Play);
    assert!(
        bird_at(&mut app).y > bird::START_Y + 20.0,
        "a bird that waited at {waiting} holds {} where a flap lifts it above {}",
        bird_at(&mut app).y,
        bird::START_Y + 20.0,
    );
}

#[test]
fn the_space_bar_flaps_as_the_word_does() {
    let mut app = app();
    tick(&mut app, 5);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    tick(&mut app, 1);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Play);
}

#[test]
fn a_bird_that_never_flaps_falls_to_the_ground_and_ends_the_run() {
    let mut app = app();
    tick(&mut app, 5);
    say(&mut app, "fly");
    tick(&mut app, 1);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Play);
    tick(&mut app, 200);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Over);
    assert_eq!(app.world().resource::<Game>().score(), 0);
    assert!(
        bird_at(&mut app).y <= world::FLOOR + bird::RADIUS + 1.0,
        "a bird that fell holds {} where the ground of {} waits",
        bird_at(&mut app).y,
        world::FLOOR,
    );
}

#[test]
fn a_bird_that_flaps_through_every_gap_scores() {
    let mut app = app();
    tick(&mut app, 5);
    say(&mut app, "fly");
    tick(&mut app, 1);
    flap_through(&mut app, 300);
    let game = app.world().resource::<Game>();
    assert_eq!(game.stage(), Stage::Play);
    assert!(
        game.score() >= 3,
        "a bird flew fifteen seconds for {} points",
        game.score(),
    );
}

#[test]
fn a_word_after_the_fall_opens_a_run_of_no_score() {
    let mut app = app();
    tick(&mut app, 5);
    say(&mut app, "fly");
    tick(&mut app, 200);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Over);
    say(&mut app, "fly");
    tick(&mut app, 1);
    let game = app.world().resource::<Game>();
    assert_eq!(game.stage(), Stage::Play);
    assert_eq!(game.score(), 0);
    assert_eq!(game.best(), 0);
    assert!((bird_at(&mut app).y - bird::START_Y).abs() < 0.5);
    let mut query = app.world_mut().query::<&Pipe>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn a_pipe_touches_a_bird_that_reaches_its_body() {
    let pipe = Pipe::at(0.0);
    let at = Vec2::ZERO;
    assert!(!pipe.touches(at, Vec2::ZERO, bird::RADIUS));
    assert!(pipe.touches(
        at,
        Vec2::new(0.0, -pipe::GAP / 2.0 + bird::RADIUS - 1.0),
        bird::RADIUS,
    ));
    assert!(pipe.touches(
        at,
        Vec2::new(0.0, pipe::GAP / 2.0 - bird::RADIUS + 1.0),
        bird::RADIUS,
    ));
    assert!(!pipe.touches(
        at,
        Vec2::new(pipe::WIDTH / 2.0 + bird::RADIUS + 1.0, 0.0),
        bird::RADIUS,
    ));
}
