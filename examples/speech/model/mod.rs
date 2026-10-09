pub mod attention;
pub mod decoder;
pub mod dims;
pub mod encoder;
pub mod parameters;

use crate::model::decoder::Decoder;
use crate::model::dims::Dims;
use crate::model::encoder::Encoder;
use crate::model::parameters::Parameters;
use crate::tokenizer;
use bevy_neura::{Inputs, Model, ModelPlan, NeuraRuntime, Role, Roles};
use neura::{Element, Run, Shape, Value};

pub const MEL: Role = "mel";
pub const TOKENS: Role = "tokens";
pub const CURSOR: Role = "cursor";
pub const TOKEN: Role = "token";
pub const LOGITS: Role = "logits";
const SILENT: f32 = 0.6;

pub struct EncoderModel {
    model: Model,
    cross: Vec<(Value<'static>, Value<'static>)>,
}

impl EncoderModel {
    pub fn load(runtime: &NeuraRuntime, dims: &Dims, weights: &[u8]) -> Self {
        let mut parameters = Parameters::new();
        let mut cross = Vec::new();
        let plan = ModelPlan::build(|graph| {
            let encoder = Encoder::build(graph, &mut parameters, dims);
            let mel = graph.input(
                Shape::of([1, dims.mel_bins, 1, dims.frames]),
                Element::Single,
            );
            let states = encoder.forward(graph, mel);
            let projected = encoder.project(graph, states);
            for (keys, values) in &projected {
                graph.retain(*keys);
                graph.retain(*values);
            }
            cross = projected;
            Roles::new().input(MEL, mel)
        });
        let model = plan.attach(runtime);
        parameters.pour(runtime, model.program(), weights);
        Self { model, cross }
    }

    pub fn run(&self, runtime: &NeuraRuntime, spectrogram: Vec<f32>) -> Run {
        self.model
            .run(runtime, &Inputs::new().write(MEL, spectrogram))
    }

    pub fn cross(&self, runtime: &NeuraRuntime) -> Vec<(Vec<f32>, Vec<f32>)> {
        self.cross
            .iter()
            .map(|(keys, values)| {
                (
                    runtime.read(self.model.program(), *keys),
                    runtime.read(self.model.program(), *values),
                )
            })
            .collect()
    }
}

pub struct DecoderModel {
    model: Model,
    cross: Vec<(Value<'static>, Value<'static>)>,
}

impl DecoderModel {
    pub fn load(runtime: &NeuraRuntime, dims: &Dims, weights: &[u8]) -> Self {
        let mut parameters = Parameters::new();
        let mut cross = Vec::new();
        let mut placements = None;
        let plan = ModelPlan::build(|graph| {
            let decoder = Decoder::build(graph, &mut parameters, dims);
            let tokens = graph.input(Shape::of([1, 1, dims.prefix(), 1]), Element::Single);
            let positions = graph.input(Shape::of([1, 1, dims.prefix(), 1]), Element::Single);
            let cursor = graph.input(Shape::of([1, 1, 1, 1]), Element::Single);
            let projected = (0..dims.decoder_layers)
                .map(|_| {
                    (
                        graph.input(
                            Shape::of([dims.heads, 1, dims.positions, dims.width()]),
                            Element::Single,
                        ),
                        graph.input(
                            Shape::of([dims.heads, 1, dims.positions, dims.width()]),
                            Element::Single,
                        ),
                    )
                })
                .collect::<Vec<(Value<'static>, Value<'static>)>>();
            let (logits, token) = decoder.forward(graph, tokens, positions, cursor, &projected);
            graph.retain(token);
            graph.retain(logits);
            placements = Some(positions);
            cross = projected;
            Roles::new()
                .input(TOKENS, tokens)
                .input(CURSOR, cursor)
                .output(TOKEN, token)
                .output(LOGITS, logits)
        });
        let model = plan.attach(runtime);
        parameters.pour(runtime, model.program(), weights);
        let placements = placements.expect("a decoder holds the placements of its prefix");
        let positions = (0..dims.prefix())
            .map(|place| place as f32)
            .collect::<Vec<f32>>();
        runtime.write(model.program(), placements, &positions);
        Self { model, cross }
    }

    pub fn carry(&self, runtime: &NeuraRuntime, cross: &[(Vec<f32>, Vec<f32>)]) {
        assert_eq!(
            cross.len(),
            self.cross.len(),
            "a decoder of {} cross attentions reads {}",
            self.cross.len(),
            cross.len(),
        );
        for ((keys, values), (key, value)) in self.cross.iter().zip(cross) {
            runtime.write(self.model.program(), *keys, key);
            runtime.write(self.model.program(), *values, value);
        }
    }

    pub fn logits(&self, runtime: &NeuraRuntime) -> Vec<f32> {
        self.model.read(runtime, LOGITS)
    }

    pub fn hears_speech(&self, runtime: &NeuraRuntime) -> bool {
        self.chance(runtime, tokenizer::NO_SPEECH) < SILENT
    }

    fn chance(&self, runtime: &NeuraRuntime, token: u32) -> f32 {
        let logits = self.logits(runtime);
        let ceiling = logits.iter().copied().fold(f32::MIN, f32::max);
        let total = logits
            .iter()
            .map(|value| (value - ceiling).exp())
            .sum::<f32>();
        (logits[token as usize] - ceiling).exp() / total
    }

    pub fn step(&self, runtime: &NeuraRuntime, tokens: Vec<f32>, cursor: u32) -> (u32, Run) {
        let run = self.model.run(
            runtime,
            &Inputs::new()
                .write(TOKENS, tokens)
                .write(CURSOR, vec![cursor as f32]),
        );
        let token = self.model.read(runtime, TOKEN);
        (token[0] as u32, run)
    }
}
