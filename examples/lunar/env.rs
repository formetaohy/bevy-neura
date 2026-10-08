use crate::body::{Body, HULL, LEFT_LEG, Part, RIGHT_LEG};
use crate::particles::Particles;
use crate::random::Random;
use crate::shape::Polygon;
use crate::terrain::{self, Terrain};
use crate::world::World;
use bevy::math::Vec2;

pub const FPS: f32 = 50.0;
pub const STEP: f32 = 1.0 / FPS;
pub const OBSERVATION: usize = 8;
pub const EPISODE_LIMIT: u32 = 1000;

const MAIN_ENGINE_POWER: f32 = 13.0;
const SIDE_ENGINE_POWER: f32 = 0.6;
const MAIN_ENGINE_OFFSET: f32 = 4.0 / terrain::SCALE;
const SIDE_ENGINE_HEIGHT: f32 = 14.0 / terrain::SCALE;
const SIDE_ENGINE_AWAY: f32 = 12.0 / terrain::SCALE;
const SIDE_ENGINE_ANCHOR: f32 = 17.0 / terrain::SCALE;
const INITIAL_RANDOM: f32 = 1000.0;
const LEG_AWAY: f32 = 20.0 / terrain::SCALE;
const LEG_DOWN: f32 = 18.0 / terrain::SCALE;
const LEG_TILT: f32 = 0.05;
const LEG_REACH: f32 = 0.4;
const LEG_HALF: Vec2 = Vec2::new(2.0 / terrain::SCALE, 8.0 / terrain::SCALE);
const HULL_DENSITY: f32 = 5.0;
const LEG_DENSITY: f32 = 1.0;
const FRICTION: f32 = 0.1;
const EXHAUST_DENSITY: f32 = 3.5;
const THRUSTER_DENSITY: f32 = 0.7;
const HULL_SHAPE: [(f32, f32); 6] = [
    (-14.0, 17.0),
    (-17.0, 0.0),
    (-17.0, -10.0),
    (17.0, -10.0),
    (17.0, 0.0),
    (14.0, 17.0),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Coast,
    Left,
    Main,
    Right,
}

impl Action {
    pub const ALL: [Action; 4] = [Self::Coast, Self::Left, Self::Main, Self::Right];

    pub fn name(self) -> &'static str {
        match self {
            Self::Coast => "coast",
            Self::Left => "left",
            Self::Main => "main",
            Self::Right => "right",
        }
    }

    pub fn of(index: usize) -> Self {
        *Self::ALL
            .get(index)
            .unwrap_or_else(|| panic!("the lander holds four engines, not the engine {index}"))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    Crashed,
    Landed,
    Departed,
}

pub struct Move {
    pub observation: [f32; OBSERVATION],
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
}

pub struct Lander {
    world: World,
    random: Random,
    particles: Particles,
    effects: bool,
    craft: usize,
    action: Action,
    thrust: (f32, f32),
    previous_shaping: Option<f32>,
    crashed: bool,
    steps: u32,
    reward: f32,
    ending: Option<Ending>,
    generation: u64,
}

impl Lander {
    pub fn new(seed: u64, effects: bool) -> Self {
        let random = Random::seeded(seed);
        let world = World::new(Terrain::generated(&mut Random::seeded(seed)));
        let mut lander = Self {
            world,
            random,
            particles: Particles::default(),
            effects,
            craft: 0,
            action: Action::Coast,
            thrust: (0.0, 0.0),
            previous_shaping: None,
            crashed: false,
            steps: 0,
            reward: 0.0,
            ending: None,
            generation: 0,
        };
        lander.reset();
        lander
    }

    pub fn reset(&mut self) {
        let terrain = Terrain::generated(&mut self.random);
        let mut world = World::new(terrain);
        let origin = Vec2::new(terrain::WIDTH / 2.0, terrain::HEIGHT);
        let mut parts = vec![Part::hull(
            Polygon::new(
                HULL_SHAPE
                    .iter()
                    .map(|(x, y)| Vec2::new(x / terrain::SCALE, y / terrain::SCALE))
                    .collect(),
            ),
            HULL_DENSITY,
        )];
        for (tag, side) in [(RIGHT_LEG, -1.0f32), (LEFT_LEG, 1.0)] {
            let tilt = side * (LEG_TILT - LEG_REACH);
            let anchor = Vec2::new(side * LEG_AWAY, LEG_DOWN);
            let center = -Vec2::from_angle(tilt).rotate(anchor);
            parts.push(Part::leg(
                tag,
                Polygon::boxed(LEG_HALF).rotated(tilt).shifted(center),
                LEG_DENSITY,
            ));
        }
        let craft = world.add(Body::at_origin(parts, origin, 0.0, FRICTION, 0.0));
        world.bodies[craft].apply_force_to_center(Vec2::new(
            self.random.uniform(-INITIAL_RANDOM, INITIAL_RANDOM),
            self.random.uniform(-INITIAL_RANDOM, INITIAL_RANDOM),
        ));
        self.world = world;
        self.particles.clear();
        self.craft = craft;
        self.action = Action::Coast;
        self.thrust = (0.0, 0.0);
        self.previous_shaping = None;
        self.crashed = false;
        self.steps = 0;
        self.reward = 0.0;
        self.ending = None;
        self.generation += 1;
        self.advance(Action::Coast);
    }

    pub fn step(&mut self, action: Action) -> Move {
        self.steps += 1;
        let observation = self.advance(action);
        let truncated = self.steps >= EPISODE_LIMIT && self.ending.is_none();
        Move {
            observation,
            reward: self.reward,
            terminated: self.ending.is_some(),
            truncated,
        }
    }

    pub fn observation(&self) -> [f32; OBSERVATION] {
        let body = &self.world.bodies[self.craft];
        let origin = body.origin();
        let velocity = body.linear;
        [
            (origin.x - terrain::WIDTH / 2.0) / (terrain::WIDTH / 2.0),
            (origin.y - (terrain::PAD_Y + LEG_DOWN)) / (terrain::HEIGHT / 2.0),
            velocity.x * (terrain::WIDTH / 2.0) / FPS,
            velocity.y * (terrain::HEIGHT / 2.0) / FPS,
            body.angle,
            20.0 * body.angular / FPS,
            if body.part_touching(RIGHT_LEG) {
                1.0
            } else {
                0.0
            },
            if body.part_touching(LEFT_LEG) {
                1.0
            } else {
                0.0
            },
        ]
    }

    pub fn terrain(&self) -> &Terrain {
        &self.world.terrain
    }

    pub fn craft(&self) -> &Body {
        &self.world.bodies[self.craft]
    }

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn thrust(&self) -> (f32, f32) {
        self.thrust
    }

    pub fn particles(&self) -> &Particles {
        &self.particles
    }
    pub fn ending(&self) -> Option<Ending> {
        self.ending
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn fade(&mut self) {
        self.particles.fade();
    }

    fn advance(&mut self, action: Action) -> [f32; OBSERVATION] {
        let body = &self.world.bodies[self.craft];
        let origin = body.origin();
        let (sine, cosine) = body.angle.sin_cos();
        let tip = Vec2::new(sine, cosine);
        let side = Vec2::new(-cosine, sine);
        let dispersion = [
            self.random.uniform(-1.0, 1.0) / terrain::SCALE,
            self.random.uniform(-1.0, 1.0) / terrain::SCALE,
        ];
        let mut main = 0.0;
        let mut lateral = 0.0;
        if action == Action::Main {
            main = 1.0;
            let offset = nozzle(
                tip,
                side,
                MAIN_ENGINE_OFFSET + 2.0 * dispersion[0],
                dispersion[1],
            );
            self.exhaust(origin + offset, offset * MAIN_ENGINE_POWER, EXHAUST_DENSITY);
            self.world.bodies[self.craft]
                .apply_impulse(-offset * (MAIN_ENGINE_POWER * main), origin + offset);
        }
        if matches!(action, Action::Left | Action::Right) {
            lateral = 1.0;
            let direction = if action == Action::Left { -1.0 } else { 1.0 };
            let offset = nozzle(
                tip,
                side,
                dispersion[0],
                3.0 * dispersion[1] + direction * SIDE_ENGINE_AWAY,
            );
            let at = origin
                + Vec2::new(
                    offset.x - tip.x * SIDE_ENGINE_ANCHOR,
                    offset.y + tip.y * SIDE_ENGINE_HEIGHT,
                );
            self.exhaust(at, offset * SIDE_ENGINE_POWER, THRUSTER_DENSITY);
            self.world.bodies[self.craft]
                .apply_impulse(-offset * (SIDE_ENGINE_POWER * lateral), at);
        }
        self.action = action;
        self.thrust = (main, lateral);
        self.world.step(STEP);
        self.particles.step(&self.world.terrain, STEP);
        let craft = &self.world.bodies[self.craft];
        self.crashed |= craft.part_touching(HULL);
        let observation = self.observation();
        let shaping = -100.0
            * (observation[0] * observation[0] + observation[1] * observation[1]).sqrt()
            - 100.0 * (observation[2] * observation[2] + observation[3] * observation[3]).sqrt()
            - 100.0 * observation[4].abs()
            + 10.0 * observation[6]
            + 10.0 * observation[7];
        let mut reward = match self.previous_shaping {
            Some(previous) => shaping - previous,
            None => 0.0,
        };
        self.previous_shaping = Some(shaping);
        reward -= main * 0.30;
        reward -= lateral * 0.03;
        if self.crashed || observation[0].abs() >= 1.0 {
            reward = -100.0;
            self.ending = Some(if self.crashed {
                Ending::Crashed
            } else {
                Ending::Departed
            });
        }
        if self.world.bodies[self.craft].asleep() {
            reward = 100.0;
            self.ending = Some(Ending::Landed);
        }
        self.reward = reward;
        observation
    }

    fn exhaust(&mut self, at: Vec2, impulse: Vec2, density: f32) {
        if self.effects {
            self.particles.spawn(at, impulse, density);
        }
    }
}

fn nozzle(tip: Vec2, side: Vec2, along: f32, lateral: f32) -> Vec2 {
    let offset = tip * along + side * lateral;
    Vec2::new(offset.x, -offset.y)
}
