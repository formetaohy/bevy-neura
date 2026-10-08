use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;

#[path = "../examples/speech/direction.rs"]
mod direction;
#[path = "../examples/speech/game.rs"]
mod game;
#[path = "../examples/speech/ghost.rs"]
mod ghost;
#[path = "../examples/speech/maze.rs"]
mod maze;
#[path = "../examples/speech/utterance.rs"]
mod utterance;
#[path = "../examples/speech/walker.rs"]
mod walker;

use direction::Direction;
use game::{Game, Pac, Stage};
use maze::{Maze, Tile};
use utterance::Utterance;
use walker::Walker;

const STEP: f32 = 0.05;

fn reachable(from: IVec2, ghost: bool) -> HashSet<IVec2> {
    let mut seen = HashSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        for step in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            let next = at + step;
            if seen.contains(&next) || !Maze::passable(at, next, ghost) {
                continue;
            }
            seen.insert(next);
            queue.push_back(next);
        }
    }
    seen
}

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Game>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<ColorMaterial>::default());
    app.insert_resource(Time::<()>::default());
    app.add_message::<Utterance>();
    app.add_systems(Startup, game::setup);
    app.add_systems(Update, (game::direct, game::advance, game::face).chain());
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

fn pac(app: &mut App) -> Walker {
    let mut query = app.world_mut().query::<&Pac>();
    query
        .single(app.world())
        .expect("a maze holds one pac")
        .walk
}

fn say(app: &mut App, text: &str) {
    app.world_mut().write_message(Utterance {
        text: text.to_string(),
    });
}

fn rotation(app: &mut App) -> f32 {
    let mut query = app.world_mut().query::<(&Pac, &Transform)>();
    query
        .single(app.world())
        .expect("a maze holds one pac")
        .1
        .rotation
        .to_euler(EulerRot::ZYX)
        .0
}

#[test]
fn the_pellets_of_a_maze_wait_behind_corridors_a_pac_can_walk() {
    let seen = reachable(Maze::start(), false);
    for (at, tile) in Maze::tiles() {
        if Maze::pellet(at).is_some() {
            assert!(
                seen.contains(&at),
                "the pellet at {at} waits behind a wall of the maze",
            );
        }
        if tile == Tile::House {
            assert!(
                !seen.contains(&at),
                "the pac walks into the house through {at}",
            );
        }
    }
}

#[test]
fn a_ghost_leaves_the_house_and_hunts_through_the_door_it_keeps() {
    let outside = reachable(IVec2::new(9, 7), true);
    for (at, tile) in Maze::tiles() {
        if tile == Tile::House {
            assert!(
                outside.contains(&at),
                "a ghost shut in at {at} has no way out",
            );
        }
        if Maze::pellet(at).is_some() {
            assert!(
                outside.contains(&at),
                "a ghost cannot reach the pellet at {at}",
            );
        }
    }
    let inside = reachable(Maze::start(), true);
    for (at, tile) in Maze::tiles() {
        if tile == Tile::House {
            assert!(
                !inside.contains(&at),
                "a ghost outside walks back into the house through {at}",
            );
        }
    }
}

#[test]
fn the_hud_names_the_direction_a_word_asks_for() {
    assert_eq!(
        Direction::read("go left").map(Direction::name),
        Some("left")
    );
    assert_eq!(Direction::read("DOWN.").map(Direction::name), Some("down"));
    assert_eq!(Direction::read("sizzling").map(Direction::name), None);
}

#[test]
fn the_mouth_of_the_pac_points_where_the_word_walks_it() {
    let mut app = app();
    tick(&mut app, 45);
    for (word, facing) in [
        ("left", Vec2::NEG_X),
        ("down", Vec2::NEG_Y),
        ("up", Vec2::Y),
        ("right", Vec2::X),
    ] {
        say(&mut app, word);
        tick(&mut app, 1);
        let mouth = Rot2::radians(rotation(&mut app)) * Vec2::NEG_Y;
        assert!(
            mouth.distance(facing) < 0.01,
            "a pac that heard {word} points its mouth at {mouth} where {facing} holds",
        );
    }
}

#[test]
fn a_spoken_word_turns_the_pac_and_the_dots_it_crosses_count() {
    let mut app = app();
    tick(&mut app, 45);
    assert_eq!(app.world().resource::<Game>().stage(), Stage::Play);
    assert_eq!(app.world().resource::<Game>().level(), 1);
    assert_eq!(app.world().resource::<Game>().score(), 0);
    let start = pac(&mut app).tile;
    say(&mut app, " Move left.");
    tick(&mut app, 5);
    assert_eq!(pac(&mut app).tile, start + IVec2::new(-1, 0));
    assert_eq!(app.world().resource::<Game>().score(), 10);
}

#[test]
fn a_word_of_nothing_leaves_the_pac_where_it_stands() {
    let mut app = app();
    tick(&mut app, 45);
    let start = pac(&mut app).tile;
    say(&mut app, "thank you for watching");
    tick(&mut app, 20);
    assert_eq!(pac(&mut app).tile, start);
    assert_eq!(app.world().resource::<Game>().score(), 0);
}

#[test]
fn a_word_for_the_lane_a_pac_came_from_turns_it_around() {
    let mut app = app();
    tick(&mut app, 45);
    say(&mut app, "left");
    tick(&mut app, 4);
    let before = pac(&mut app).position();
    say(&mut app, "right");
    tick(&mut app, 1);
    let after = pac(&mut app).position();
    assert!(
        after.x > before.x,
        "a pac of {before} walks on to the left where the word asks for the right",
    );
    assert!(
        after.x - before.x < 1.0,
        "a pac jumps from {before} to {after} instead of turning around where it stands",
    );
    tick(&mut app, 8);
    assert!(
        pac(&mut app).position().x > Maze::start().x as f32,
        "a pac that turned around never walks past the tile it came from",
    );
}

#[test]
fn a_word_the_maze_cannot_walk_leaves_the_pac_waiting() {
    let mut app = app();
    tick(&mut app, 45);
    say(&mut app, "right");
    tick(&mut app, 4);
    assert_eq!(pac(&mut app).tile, IVec2::new(11, 13));
    say(&mut app, "up");
    tick(&mut app, 5);
    assert_eq!(
        pac(&mut app).position(),
        Vec2::new(12.0, 13.0),
        "a pac of {:?} walks on where the word asks for the wall above it",
        pac(&mut app).position(),
    );
    say(&mut app, "right");
    tick(&mut app, 4);
    assert_eq!(pac(&mut app).tile, IVec2::new(13, 13));
}

#[test]
fn a_wall_stops_the_pac_and_the_next_word_turns_it() {
    let mut app = app();
    tick(&mut app, 45);
    say(&mut app, "down");
    tick(&mut app, 20);
    assert_eq!(pac(&mut app).tile, Maze::start());
    say(&mut app, "left");
    tick(&mut app, 20);
    assert_eq!(pac(&mut app).tile, Maze::start() + IVec2::new(-5, 0));
}

#[test]
fn a_word_after_the_last_life_opens_a_new_maze() {
    let mut app = app();
    let mut ticks = 0;
    while app.world().resource::<Game>().stage() != Stage::Over && ticks < 3000 {
        tick(&mut app, 20);
        ticks += 20;
    }
    assert_eq!(
        app.world().resource::<Game>().stage(),
        Stage::Over,
        "a pac that never moves is never caught three times",
    );
    say(&mut app, "left");
    tick(&mut app, 1);
    let game = app.world().resource::<Game>();
    assert!(matches!(game.stage(), Stage::Ready(_)));
    assert_eq!(game.level(), 1);
    assert_eq!(game.lives(), 3);
    assert_eq!(game.score(), 0);
    assert_eq!(pac(&mut app).tile, Maze::start());
}

#[test]
fn a_ghost_that_catches_a_pac_that_stands_still_takes_a_life() {
    let mut app = app();
    tick(&mut app, 800);
    assert!(
        app.world().resource::<Game>().lives() < 3,
        "a pac that stands still for forty seconds never met a ghost",
    );
    assert_eq!(pac(&mut app).tile, Maze::start());
}
