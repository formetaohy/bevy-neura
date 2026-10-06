use bevy::prelude::*;
use bevy_neura::{Inputs, Model, ModelPlan, NeuraPlugin, NeuraRuntime, Roles};
use neura::{AdamW, Checkpoint, Element, Init, Linear, Shape, mse_loss};

const VALUES: [f32; 4] = [0.25, -0.5, 1.5, 2.0];
const TARGETS: [f32; 4] = [-1.0, 0.5, 2.0, -1.5];

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app
}

fn updating() -> ModelPlan {
    ModelPlan::build(|graph| {
        let dense = Linear::new(
            graph,
            "dense",
            2,
            2,
            Init::Uniform {
                low: -0.4,
                high: 0.4,
            },
            Element::Single,
        );
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let target = graph.input(Shape::matrix(2, 2), Element::Single);
        let loss = mse_loss(graph, dense.forward(graph, observation), target);
        let gradients = graph.backward(loss);
        let mut optimizer = AdamW::new(graph, "optimizer", 0.05, 0.9, 0.999, 1e-8, 0.0);
        optimizer.track_all(graph, &dense.parameters());
        optimizer.step(graph, &gradients);
        Roles::new()
            .input("observation", observation)
            .input("target", target)
            .output("loss", loss)
    })
}

fn reading() -> ModelPlan {
    ModelPlan::build(|graph| {
        let dense = Linear::new(graph, "dense", 2, 2, Init::Zero, Element::Single);
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let prediction = dense.forward(graph, observation);
        Roles::new()
            .input("observation", observation)
            .output("prediction", prediction)
    })
}

fn prediction(runtime: &NeuraRuntime, model: &Model) -> Vec<f32> {
    model.run(runtime, &Inputs::new().write("observation", VALUES));
    model.read(runtime, "prediction")
}

fn distance(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f32::max)
}

#[test]
fn a_checkpoint_returns_the_weights_a_shared_store_held() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let learner = updating().attach(runtime);
    let reader = reading().share(runtime, &learner);
    let before = prediction(runtime, &reader);
    let checkpoint = learner.checkpoint(runtime);
    let inputs = Inputs::new()
        .write("observation", VALUES)
        .write("target", TARGETS);
    for _ in 0..8 {
        learner.run(runtime, &inputs);
    }
    let moved = prediction(runtime, &reader);
    assert!(
        distance(&before, &moved) > 1e-4,
        "eight runs move the prediction a reader shares",
    );
    learner.restore(runtime, &Checkpoint::decode(checkpoint.bytes()));
    let after = prediction(runtime, &reader);
    assert!(
        distance(&before, &after) < 1e-6,
        "the checkpoint returns the prediction the shared store held",
    );
}
