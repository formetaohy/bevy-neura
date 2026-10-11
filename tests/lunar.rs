#[path = "../examples/lunar/body.rs"]
mod body;
#[path = "../examples/lunar/contact.rs"]
mod contact;
#[path = "../examples/lunar/env.rs"]
mod env;
#[path = "../examples/lunar/learner.rs"]
mod learner;
#[path = "../examples/lunar/net.rs"]
mod net;
#[path = "../examples/lunar/particles.rs"]
mod particles;
#[path = "../examples/lunar/policy.rs"]
mod policy;
#[path = "../examples/lunar/random.rs"]
mod random;
#[path = "../examples/lunar/rollout.rs"]
mod rollout;
#[path = "../examples/lunar/shape.rs"]
mod shape;
#[path = "../examples/lunar/terrain.rs"]
mod terrain;
#[path = "../examples/lunar/train.rs"]
mod train;
#[path = "../examples/lunar/world.rs"]
mod world;

use bevy::prelude::*;
use bevy_neura::{NeuraPlugin, NeuraRuntime};
use body::{Body, Part};
use env::{Action, Ending, Lander, OBSERVATION, STEP};
use shape::Polygon;
use terrain::Terrain;
use world::{GRAVITY, World};

const SCRIPT: [Action; 8] = [
    Action::Coast,
    Action::Main,
    Action::Coast,
    Action::Main,
    Action::Left,
    Action::Coast,
    Action::Right,
    Action::Main,
];

struct Played {
    reward: f32,
    expected: f32,
    steps: u32,
    ending: Option<Ending>,
    first: [f32; OBSERVATION],
    last: [f32; OBSERVATION],
}

fn play(seed: u64) -> Played {
    let mut lander = Lander::new(seed, false);
    let first = lander.observation();
    let (mut reward, mut expected, mut steps) = (0.0, 0.0, 0);
    let mut carried = shaping_of(&first);
    loop {
        let action = SCRIPT[steps as usize % SCRIPT.len()];
        let moved = lander.step(action);
        let reached = shaping_of(&moved.observation);
        let fuel = match action {
            Action::Main => 0.30,
            Action::Left | Action::Right => 0.03,
            Action::Coast => 0.0,
        };
        expected += match moved.terminated {
            true => match lander.ending() {
                Some(Ending::Landed) => 100.0,
                _ => -100.0,
            },
            false => reached - carried - fuel,
        };
        carried = reached;
        reward += moved.reward;
        steps += 1;
        if moved.terminated || moved.truncated {
            return Played {
                reward,
                expected,
                steps,
                ending: lander.ending(),
                first,
                last: moved.observation,
            };
        }
    }
}

fn shaping_of(state: &[f32; OBSERVATION]) -> f32 {
    -100.0 * (state[0] * state[0] + state[1] * state[1]).sqrt()
        - 100.0 * (state[2] * state[2] + state[3] * state[3]).sqrt()
        - 100.0 * state[4].abs()
        + 10.0 * state[6]
        + 10.0 * state[7]
}

#[test]
fn every_step_of_an_episode_scores_its_shaping_its_fuel_or_its_end() {
    for seed in [0, 5, 11] {
        let played = play(seed);
        assert!(
            (played.reward - played.expected).abs() < 0.05,
            "the episode of seed {seed} scores {} where its shaping, fuel and end read {}",
            played.reward,
            played.expected,
        );
    }
}

#[test]
fn a_body_of_the_sky_falls_at_the_gravity_of_the_moon() {
    let mut world = World::new(Terrain::generated(&mut random::Random::seeded(7)));
    let body = world.add(Body::at_origin(
        vec![Part::hull(
            Polygon::boxed(bevy::math::Vec2::splat(0.2)),
            1.0,
        )],
        bevy::math::Vec2::new(10.0, terrain::HEIGHT + 4.0),
        0.0,
        0.1,
        0.0,
    ));
    let steps = 20;
    for _ in 0..steps {
        world.step(STEP);
    }
    let fallen = world.bodies[body].linear.y;
    let expected = steps as f32 * STEP * GRAVITY.y;
    assert!(
        (fallen - expected).abs() < 1e-3,
        "{steps} steps of free fall reach {fallen} where gravity carries {expected}",
    );
}

#[test]
fn the_main_engine_lifts_the_lander() {
    let mut coasting = Lander::new(4, false);
    let mut thrusting = Lander::new(4, false);
    for _ in 0..10 {
        coasting.step(Action::Coast);
        thrusting.step(Action::Main);
    }
    let (coasted, lifted) = (coasting.observation()[3], thrusting.observation()[3]);
    assert!(
        lifted > coasted + 0.3,
        "ten steps of thrust climb at {lifted} where ten steps of coast fall at {coasted}",
    );
}

#[test]
fn the_engines_of_the_lander_steer_it_sideways() {
    let mut left = Lander::new(9, false);
    let mut right = Lander::new(9, false);
    for _ in 0..12 {
        left.step(Action::Left);
        right.step(Action::Right);
    }
    let (leftwards, rightwards) = (left.observation()[5], right.observation()[5]);
    assert!(
        leftwards > 0.0 && rightwards < 0.0,
        "a left engine of {leftwards} and a right engine of {rightwards} turn one way",
    );
}

#[test]
fn the_observation_of_a_flight_stays_within_the_space_of_the_lander() {
    let mut lander = Lander::new(2, false);
    let mut step = 0;
    loop {
        let moved = lander.step(SCRIPT[step as usize % SCRIPT.len()]);
        for (index, value) in moved.observation.iter().enumerate() {
            assert!(
                value.is_finite() && (index >= 5 || value.abs() < 2.5),
                "the observation {index} of a flight reads {value}",
            );
        }
        step += 1;
        if moved.terminated || moved.truncated {
            break;
        }
    }
}

#[test]
fn the_same_seed_plays_the_same_episode() {
    let first = play(3);
    let second = play(3);
    assert_eq!(first.steps, second.steps);
    assert_eq!(first.reward, second.reward);
    assert_eq!(first.ending, second.ending);
    assert_eq!(first.first, second.first);
    assert_eq!(first.last, second.last);
}

#[test]
fn the_engines_of_the_lander_leave_a_trail() {
    let mut lander = Lander::new(5, true);
    lander.step(Action::Main);
    assert_eq!(lander.action(), Action::Main);
    assert_eq!(lander.thrust(), (1.0, 0.0));
    let puffs = lander.particles().iter().count();
    assert_eq!(puffs, 1, "a step of the main engine leaves {puffs} puffs");
    lander.fade();
    assert_eq!(
        lander.particles().iter().count(),
        1,
        "a puff fades after a frame"
    );
    for _ in 0..8 {
        lander.fade();
    }
    assert_eq!(
        lander.particles().iter().count(),
        0,
        "a puff outlives eight frames"
    );
}

#[test]
fn a_rollout_carries_a_reward_back_to_its_landing() {
    let mut rollout = rollout::Rollout::default();
    for row in 0..rollout::ROWS as usize {
        let last = row == (rollout::STEPS as usize - 1) * rollout::ENVS as usize;
        rollout.record(rollout::Record {
            observation: [0.0; OBSERVATION],
            following: [0.0; OBSERVATION],
            action: 0.0,
            log_probability: 0.0,
            value: 0.0,
            reward: if last { 1.0 } else { 0.0 },
            terminated: false,
            truncated: false,
        });
    }
    assert!(rollout.full());
    rollout.finish(&vec![0.0; rollout::ENVS as usize], 1.0, 1.0);
    let targets = rollout.targets();
    for step in 0..rollout::STEPS as usize {
        let row = step * rollout::ENVS as usize;
        assert_eq!(
            targets[row], 1.0,
            "the reward of the last step reaches step {step} as {}",
            targets[row],
        );
        assert_eq!(
            targets[row + 1],
            0.0,
            "a reward of one aim spreads sideways"
        );
    }
    let mut batch = rollout::Batch::default();
    rollout.gather(&mut batch, rollout.order());
    let sum = batch.advantages.iter().sum::<f32>();
    assert!(
        sum.abs() < 1e-3,
        "advantages of {sum} carry a bias into the policy",
    );
}

#[test]
fn a_drawn_action_of_the_policy_matches_the_logarithm_it_reports() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    let runtime = app.world().resource::<NeuraRuntime>().clone();
    let learner = learner::Learner::attach(&runtime);
    let sampler = policy::Sampler::share(&runtime, learner.model(), 1);
    let greedy = policy::Greedy::share(&runtime, learner.model());
    let observation = [0.1, 0.2, -0.3, 0.4, 0.05, -0.1, 1.0, 0.0];
    let sample = sampler.sample(&runtime, &observation, 1.5);
    let guess = greedy.choose(&runtime, &observation);
    let highest = guess.logits.iter().copied().fold(f32::MIN, f32::max);
    let total = guess
        .logits
        .iter()
        .map(|logit| (logit - highest).exp())
        .sum::<f32>();
    let drawn = sample.actions[0] as usize;
    let expected = guess.logits[drawn] - highest - total.ln();
    assert!(
        (sample.log_probabilities[0] - expected).abs() < 1e-3,
        "a drawn action {} reports {} where its scores read {expected}",
        sample.log_probabilities[0],
        expected,
    );
    assert!(sample.values[0].is_finite());
    let favoured = guess
        .logits
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.partial_cmp(right.1).expect("a score is a number"))
        .map(|(at, _)| at)
        .expect("a policy scores at least one action");
    assert_eq!(
        guess.action,
        Action::of(favoured),
        "a greedy policy answers {} where its scores favour {}",
        guess.action.name(),
        Action::of(favoured).name(),
    );
}

fn build(runtime: Res<NeuraRuntime>, mut commands: Commands) {
    let learner = learner::Learner::attach(&runtime);
    let sampler = policy::Sampler::share(&runtime, learner.model(), rollout::ENVS);
    commands.insert_resource(train::Training::of(learner, sampler));
}

#[test]
fn the_training_lifts_the_policy_of_the_lander() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default())
        .init_resource::<Time>()
        .add_systems(Startup, build)
        .add_systems(Update, train::advance);
    app.update();
    let runtime = app.world().resource::<NeuraRuntime>().clone();
    app.world_mut().resource_mut::<train::Training>().steps = train::TURBO;
    while app.world().resource::<train::Training>().samples < 300_000 {
        app.update();
    }
    let training = app.world().resource::<train::Training>();
    assert!(
        training.best > 150.0,
        "{} samples of training never land the lander, best {}",
        training.samples,
        training.best,
    );
    assert!(
        training.mean() > -200.0,
        "{} samples of training hold a mean of {}",
        training.samples,
        training.mean(),
    );
    let greedy = policy::Greedy::share(&runtime, training.model());
    let guess = greedy.choose(&runtime, &[0.0; OBSERVATION]);
    assert!(guess.value.is_finite());
}

#[test]
fn the_craft_rests_on_its_legs_and_not_on_its_hull() {
    let lander = Lander::new(7, false);
    let body = lander.craft();
    let lowest = |part: &body::Part| {
        part.polygon
            .vertices()
            .iter()
            .map(|vertex| vertex.y)
            .fold(f32::INFINITY, f32::min)
    };
    let hull = lowest(body.part(body::HULL));
    for tag in [body::LEFT_LEG, body::RIGHT_LEG] {
        let foot = lowest(body.part(tag));
        assert!(
            foot < hull - 0.2,
            "the foot of the leg {tag} reaches {foot} where the bottom of the hull stands at {hull}",
        );
    }
}

#[test]
fn the_pad_of_the_moon_is_flat_and_the_terrain_of_an_episode_renews() {
    let mut lander = Lander::new(11, false);
    let generation = lander.generation();
    let mut heights = Vec::new();
    for at in [
        terrain::PAD_X1,
        (terrain::PAD_X1 + terrain::PAD_X2) / 2.0,
        terrain::PAD_X2,
    ] {
        heights.push(
            lander
                .terrain()
                .closest(bevy::math::Vec2::new(at, terrain::HEIGHT))
                .point
                .y,
        );
    }
    for height in &heights {
        assert!(
            (height - heights[0]).abs() < 1e-4,
            "the pad of the moon is not flat: {heights:?}",
        );
        assert!(
            (height - terrain::PAD_Y).abs() < 0.05,
            "the pad stands at {height} where its flags stand at {}",
            terrain::PAD_Y,
        );
    }
    assert!(lander.terrain().points().len() >= 3);
    lander.reset();
    assert_eq!(lander.generation(), generation + 1);
}

#[test]
fn every_engine_of_the_lander_answers_to_its_own_name() {
    let mut names = Action::ALL.map(Action::name).to_vec();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "two engines answer to one name");
}
