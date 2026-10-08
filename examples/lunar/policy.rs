use crate::env::{Action, OBSERVATION};
use crate::net::Net;
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{Element, Shape};

pub const OBSERVATION_ROLE: Role = "observation";
pub const SEED_ROLE: Role = "seed";
pub const ACTION_ROLE: Role = "action";
pub const LOGITS_ROLE: Role = "logits";
pub const VALUE_ROLE: Role = "value";
pub const LOG_PROBABILITY_ROLE: Role = "log_probability";
pub const ADVANTAGE_ROLE: Role = "advantage";
pub const RETURN_ROLE: Role = "return";
pub const POLICY_LOSS_ROLE: Role = "policy_loss";
pub const VALUE_LOSS_ROLE: Role = "value_loss";
pub const ENTROPY_ROLE: Role = "entropy";
pub const LOSS_ROLE: Role = "loss";
const ROWS_ROLE: Role = "rows";

pub const ACTIONS: usize = Action::ALL.len();

pub struct Sample {
    pub actions: Vec<f32>,
    pub log_probabilities: Vec<f32>,
    pub values: Vec<f32>,
}

pub struct Guess {
    pub action: Action,
    pub value: f32,
    pub logits: [f32; ACTIONS],
}

pub struct Sampler {
    model: Model,
    rows: u32,
}

impl Sampler {
    pub fn share(runtime: &NeuraRuntime, source: &Model, rows: u32) -> Self {
        let bound = rows;
        let model = ModelPlan::build(|graph| {
            let rows = graph.free(bound);
            let net = Net::build(graph);
            let observation = graph.input(
                Shape::matrix(bound, OBSERVATION as u32).freed(&[(2, rows)]),
                Element::Single,
            );
            let seed = graph.input(Shape::scalar(), Element::Single);
            let (logits, value) = net.forward(graph, observation);
            let action = graph.categorical(logits, seed);
            let log_probability = graph.sum_rows(graph.mul(
                graph.one_hot(action, ACTIONS as u32),
                graph.log_softmax(logits),
            ));
            for output in [action, log_probability, value] {
                graph.retain(output);
            }
            Roles::new()
                .axis(ROWS_ROLE, rows)
                .input(OBSERVATION_ROLE, observation)
                .input(SEED_ROLE, seed)
                .output(ACTION_ROLE, action)
                .output(LOG_PROBABILITY_ROLE, log_probability)
                .output(VALUE_ROLE, value)
        })
        .share(runtime, source);
        Self { model, rows }
    }

    pub fn sample(&self, runtime: &NeuraRuntime, observations: &[f32], seed: f32) -> Sample {
        assert_eq!(
            observations.len(),
            self.rows as usize * OBSERVATION,
            "a sampler of {} rows walks {} observations",
            self.rows,
            observations.len(),
        );
        self.model
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, self.rows));
        self.model.run(
            runtime,
            &Inputs::new()
                .write(OBSERVATION_ROLE, observations)
                .write(SEED_ROLE, vec![seed]),
        );
        let mut read = self
            .model
            .pull(runtime, &[ACTION_ROLE, LOG_PROBABILITY_ROLE, VALUE_ROLE])
            .collect()
            .into_iter();
        Sample {
            actions: read.next().expect("a sampler reads the action it drew"),
            log_probabilities: read
                .next()
                .expect("a sampler reads the logarithm of the draw"),
            values: read.next().expect("a sampler reads the worth of a state"),
        }
    }

    pub fn value(&self, runtime: &NeuraRuntime, observations: &[f32], seed: f32) -> Vec<f32> {
        assert_eq!(
            observations.len() % OBSERVATION,
            0,
            "a sampler walks whole observations, not {} numbers",
            observations.len(),
        );
        let rows = observations.len() as u32 / OBSERVATION as u32;
        self.model
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, rows));
        self.model.run(
            runtime,
            &Inputs::new()
                .write(OBSERVATION_ROLE, observations)
                .write(SEED_ROLE, vec![seed]),
        );
        self.model.read(runtime, VALUE_ROLE)
    }
}

pub struct Greedy {
    model: Model,
}

impl Greedy {
    pub fn share(runtime: &NeuraRuntime, source: &Model) -> Self {
        let model = ModelPlan::build(|graph| {
            let net = Net::build(graph);
            let observation = graph.input(Shape::matrix(1, OBSERVATION as u32), Element::Single);
            let (logits, value) = net.forward(graph, observation);
            let action = graph.argmax(logits);
            for output in [action, value, logits] {
                graph.retain(output);
            }
            Roles::new()
                .input(OBSERVATION_ROLE, observation)
                .output(ACTION_ROLE, action)
                .output(LOGITS_ROLE, logits)
                .output(VALUE_ROLE, value)
        })
        .share(runtime, source);
        Self { model }
    }

    pub fn choose(&self, runtime: &NeuraRuntime, observation: &[f32; OBSERVATION]) -> Guess {
        self.model
            .run(runtime, &Inputs::new().write(OBSERVATION_ROLE, observation));
        let mut read = self
            .model
            .pull(runtime, &[ACTION_ROLE, LOGITS_ROLE, VALUE_ROLE])
            .collect()
            .into_iter();
        let action = read.next().expect("a policy answers the action it prefers");
        let logits = read
            .next()
            .expect("a policy answers the score of every action");
        let value = read.next().expect("a critic answers the worth of a state");
        Guess {
            action: Action::of(action[0] as usize),
            value: value[0],
            logits: logits.try_into().unwrap_or_else(|logits: Vec<f32>| {
                panic!(
                    "a policy of {ACTIONS} actions scores {} of them",
                    logits.len()
                )
            }),
        }
    }
}
