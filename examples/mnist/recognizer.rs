use crate::digit;
use crate::lenet::{CLASSES, Lenet};
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{Element, Shape};

pub const BOUND: u32 = 1000;
const CLASS_WIDTH: usize = CLASSES as usize;
pub const DIGITS_ROLE: Role = "digits";
pub const LOGITS_ROLE: Role = "logits";
const BATCH_ROLE: Role = "batch";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Guess {
    pub digit: u8,
    pub confidence: f32,
}

impl Guess {
    fn of(logits: &[f32; CLASS_WIDTH]) -> Self {
        let peak = logits.iter().copied().fold(f32::MIN, f32::max);
        let weighed = logits
            .iter()
            .map(|value| (value - peak).exp())
            .collect::<Vec<f32>>();
        let total = weighed.iter().sum::<f32>();
        let (digit, mass) = weighed
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .expect("a guess weighs at least one class");
        Self {
            digit: digit as u8,
            confidence: mass / total,
        }
    }
}

pub struct Recognizer {
    model: Model,
}

impl Recognizer {
    pub fn share(runtime: &NeuraRuntime, source: &Model) -> Self {
        let side = digit::SIDE as u32;
        Self {
            model: ModelPlan::build(|graph| {
                let batch = graph.free(BOUND);
                let net = Lenet::build(graph);
                let digits = graph.input(
                    Shape::of([BOUND, 1, side, side]).freed(&[(0, batch)]),
                    Element::Single,
                );
                let logits = net.forward(graph, digits, batch);
                graph.retain(logits);
                Roles::new()
                    .axis(BATCH_ROLE, batch)
                    .input(DIGITS_ROLE, digits)
                    .output(LOGITS_ROLE, logits)
            })
            .share(runtime, source),
        }
    }

    pub fn read(&self, runtime: &NeuraRuntime, digits: &[f32]) -> Vec<Guess> {
        let side = digit::SIDE * digit::SIDE;
        assert_eq!(
            digits.len() % side,
            0,
            "{} numbers hold no whole digit of {side} pixels",
            digits.len(),
        );
        let rows = (digits.len() / side) as u32;
        assert!(
            rows <= BOUND,
            "a reading of {rows} digits outruns the {BOUND} a binding names",
        );
        self.model
            .bind(runtime, &Extents::new().axis(BATCH_ROLE, rows));
        self.model
            .run(runtime, &Inputs::new().write(DIGITS_ROLE, digits));
        let logits = self.model.read(runtime, LOGITS_ROLE);
        let (guesses, rest) = logits.as_chunks::<CLASS_WIDTH>();
        assert!(
            rest.is_empty(),
            "a reading answers {CLASS_WIDTH} classes a row, and {} numbers are left over",
            rest.len(),
        );
        guesses.iter().map(Guess::of).collect()
    }
}
