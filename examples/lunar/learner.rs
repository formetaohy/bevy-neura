use crate::env::OBSERVATION;
use crate::net::Net;
use crate::policy::{
    ACTION_ROLE, ACTIONS, ADVANTAGE_ROLE, ENTROPY_ROLE, LOG_PROBABILITY_ROLE, LOSS_ROLE,
    OBSERVATION_ROLE, POLICY_LOSS_ROLE, RETURN_ROLE, VALUE_LOSS_ROLE,
};
use crate::random::Random;
use crate::rollout::{Batch, ROWS, Rollout};
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{AdamW, Element, Shape, mse_loss};

pub const MINIBATCH: u32 = 512;
pub const EPOCHS: u32 = 4;
pub const GAMMA: f32 = 0.99;
pub const LAMBDA: f32 = 0.95;
const CLIP: f32 = 0.2;
const RATE: f32 = 3e-4;
const VALUE_COEF: f32 = 0.5;
const ENTROPY_COEF: f32 = 0.01;
const ROWS_ROLE: Role = "rows";
const ROW_AXIS: u32 = 2;

#[derive(Clone, Copy, Default)]
pub struct Diagnostics {
    pub loss: f32,
    pub policy: f32,
    pub value: f32,
    pub entropy: f32,
}

pub struct Learner {
    model: Model,
    batch: Batch,
}

impl Learner {
    pub fn attach(runtime: &NeuraRuntime) -> Self {
        Self {
            model: ModelPlan::build(|graph| {
                let rows = graph.free(ROWS);
                let net = Net::build(graph);
                let observation = graph.input(
                    Shape::matrix(ROWS, OBSERVATION as u32).freed(&[(2, rows)]),
                    Element::Single,
                );
                let action =
                    graph.input(Shape::matrix(ROWS, 1).freed(&[(2, rows)]), Element::Single);
                let advantage =
                    graph.input(Shape::matrix(ROWS, 1).freed(&[(2, rows)]), Element::Single);
                let target =
                    graph.input(Shape::matrix(ROWS, 1).freed(&[(2, rows)]), Element::Single);
                let behaviour =
                    graph.input(Shape::matrix(ROWS, 1).freed(&[(2, rows)]), Element::Single);
                let (logits, value) = net.forward(graph, observation);
                let logarithm = graph.log_softmax(logits);
                let taken =
                    graph.sum_rows(graph.mul(graph.one_hot(action, ACTIONS as u32), logarithm));
                let ratio = graph.exp(graph.sub(taken, behaviour));
                let one = graph.fill(Shape::scalar(), 1.0);
                let clip = graph.fill(Shape::scalar(), CLIP);
                let clamped =
                    graph.min(graph.max(ratio, graph.sub(one, clip)), graph.add(one, clip));
                let weight = graph.select(
                    graph.greater(advantage, graph.fill(Shape::scalar(), 0.0)),
                    clamped,
                    ratio,
                );
                let policy = graph.neg(graph.mean_axis(graph.mul(weight, advantage), ROW_AXIS));
                let entropy = graph.mean_axis(
                    graph.neg(graph.sum_rows(graph.mul(graph.softmax(logits), logarithm))),
                    ROW_AXIS,
                );
                let value_loss = mse_loss(graph, value, target);
                let loss = graph.sub(
                    graph.add(
                        policy,
                        graph.mul(value_loss, graph.fill(Shape::scalar(), VALUE_COEF)),
                    ),
                    graph.mul(entropy, graph.fill(Shape::scalar(), ENTROPY_COEF)),
                );
                let gradients = graph.backward(loss);
                let mut optimizer = AdamW::new(graph, "optimizer", RATE, 0.9, 0.999, 1e-8, 0.0);
                optimizer.track_all(graph, &net.parameters());
                optimizer.step(graph, &gradients);
                for output in [loss, policy, value_loss, entropy] {
                    graph.retain(output);
                }
                Roles::new()
                    .axis(ROWS_ROLE, rows)
                    .input(OBSERVATION_ROLE, observation)
                    .input(ACTION_ROLE, action)
                    .input(ADVANTAGE_ROLE, advantage)
                    .input(RETURN_ROLE, target)
                    .input(LOG_PROBABILITY_ROLE, behaviour)
                    .output(LOSS_ROLE, loss)
                    .output(POLICY_LOSS_ROLE, policy)
                    .output(VALUE_LOSS_ROLE, value_loss)
                    .output(ENTROPY_ROLE, entropy)
            })
            .attach(runtime),
            batch: Batch::default(),
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn update(
        &mut self,
        runtime: &NeuraRuntime,
        rollout: &mut Rollout,
        random: &mut Random,
    ) -> Diagnostics {
        let minibatches = ROWS / MINIBATCH;
        self.model
            .bind(runtime, &Extents::new().axis(ROWS_ROLE, MINIBATCH));
        let (mut total, mut taken) = (Diagnostics::default(), 0.0);
        for _ in 0..EPOCHS {
            rollout.shuffle(random);
            for index in 0..minibatches {
                let rows = &rollout.order()
                    [(index * MINIBATCH) as usize..((index + 1) * MINIBATCH) as usize];
                rollout.gather(&mut self.batch, rows);
                self.model.run(
                    runtime,
                    &Inputs::new()
                        .write(OBSERVATION_ROLE, self.batch.observations.as_slice())
                        .write(ACTION_ROLE, self.batch.actions.as_slice())
                        .write(ADVANTAGE_ROLE, self.batch.advantages.as_slice())
                        .write(RETURN_ROLE, self.batch.targets.as_slice())
                        .write(
                            LOG_PROBABILITY_ROLE,
                            self.batch.log_probabilities.as_slice(),
                        ),
                );
                let read = self
                    .model
                    .read_all(runtime)
                    .into_iter()
                    .collect::<std::collections::HashMap<Role, Vec<f32>>>();
                total.loss += read.get(LOSS_ROLE).expect("a step answers its loss")[0];
                total.policy += read
                    .get(POLICY_LOSS_ROLE)
                    .expect("a step answers its policy loss")[0];
                total.value += read
                    .get(VALUE_LOSS_ROLE)
                    .expect("a step answers its value loss")[0];
                total.entropy += read
                    .get(ENTROPY_ROLE)
                    .expect("a step answers the entropy it keeps")[0];
                taken += 1.0;
            }
        }
        Diagnostics {
            loss: total.loss / taken,
            policy: total.policy / taken,
            value: total.value / taken,
            entropy: total.entropy / taken,
        }
    }
}
