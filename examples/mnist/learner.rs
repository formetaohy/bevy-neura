use crate::digit;
use crate::lenet::Lenet;
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{AdamW, Element, Shape, cross_entropy};

pub const BATCH: u32 = 128;
pub const DIGITS_ROLE: Role = "digits";
pub const ANSWERS_ROLE: Role = "answers";
pub const LOSS_ROLE: Role = "loss";
pub const RATE_ROLE: Role = "rate";
const BATCH_ROLE: Role = "batch";
const RATE: f32 = 1e-3;

pub struct Learner {
    model: Model,
}

impl Learner {
    pub fn attach(runtime: &NeuraRuntime) -> Self {
        let side = digit::SIDE as u32;
        Self {
            model: ModelPlan::build(|graph| {
                let batch = graph.free(BATCH);
                let net = Lenet::build(graph);
                let digits = graph.input(
                    Shape::of([BATCH, 1, side, side]).freed(&[(0, batch)]),
                    Element::Single,
                );
                let answers = graph.input(
                    Shape::of([BATCH, 1, 1, 1]).freed(&[(0, batch)]),
                    Element::Single,
                );
                let loss = cross_entropy(graph, net.forward(graph, digits, batch), answers);
                let gradients = graph.backward(loss);
                let mut optimizer = AdamW::new(graph, "optimizer", RATE, 0.9, 0.999, 1e-8, 0.0);
                optimizer.track_all(graph, &net.parameters());
                optimizer.step(graph, &gradients);
                graph.retain(loss);
                Roles::new()
                    .axis(BATCH_ROLE, batch)
                    .input(DIGITS_ROLE, digits)
                    .input(ANSWERS_ROLE, answers)
                    .input(RATE_ROLE, optimizer.rate())
                    .output(LOSS_ROLE, loss)
            })
            .attach(runtime),
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn step(&self, runtime: &NeuraRuntime, digits: &[f32], answers: &[f32], rate: f32) -> f32 {
        assert!(
            rate > 0.0 && rate.is_finite(),
            "a descent rate of {rate} moves nothing",
        );
        self.model
            .bind(runtime, &Extents::new().axis(BATCH_ROLE, BATCH));
        self.model.run(
            runtime,
            &Inputs::new()
                .write(DIGITS_ROLE, digits)
                .write(ANSWERS_ROLE, answers)
                .write(RATE_ROLE, [rate]),
        );
        self.model
            .read(runtime, LOSS_ROLE)
            .first()
            .copied()
            .expect("a step answers its loss")
    }
}
