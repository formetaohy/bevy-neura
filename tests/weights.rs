use bevy::prelude::*;
use bevy_neura::{Extents, Inputs, ModelPlan, NeuraPlugin, NeuraRuntime, Roles};
use neura::{AdamW, Element, Init, Linear, Sgd, Shape, mse_loss};

const VALUES: [f32; 4] = [0.5, 1.5, -1.0, 2.0];
const TARGETS: [f32; 4] = [1.0, 1.0, -1.0, 0.5];
const RATE: f32 = 0.1;
const START: f32 = 0.5;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app
}

fn scalar() -> ModelPlan {
    ModelPlan::build(|graph| {
        let weight = graph.parameter(Shape::scalar(), Init::Constant(START), Element::Single);
        let observation = graph.input(Shape::matrix(4, 1), Element::Single);
        let target = graph.input(Shape::matrix(4, 1), Element::Single);
        let loss = mse_loss(graph, graph.mul(observation, weight), target);
        let gradients = graph.backward(loss);
        let mut optimizer = Sgd::new(graph, RATE, 0.0);
        optimizer.track(graph, weight);
        optimizer.step(graph, &gradients);
        Roles::new()
            .input("observation", observation)
            .input("target", target)
            .output("weight", weight)
            .output("loss", loss)
    })
}

fn dynamic() -> ModelPlan {
    ModelPlan::build(|graph| {
        let batch = graph.free(16);
        let dense = Linear::new(
            graph,
            "dense",
            2,
            1,
            Init::Uniform {
                low: -0.5,
                high: 0.5,
            },
            Element::Single,
        );
        let observation = graph.input(Shape::matrix(16, 2).freed(&[(2, batch)]), Element::Single);
        let target = graph.input(Shape::matrix(16, 1).freed(&[(2, batch)]), Element::Single);
        let loss = mse_loss(graph, dense.forward(graph, observation), target);
        let gradients = graph.backward(loss);
        let mut optimizer = AdamW::new(graph, "optimizer", 0.02, 0.9, 0.999, 1e-8, 0.0);
        optimizer.track_all(graph, &dense.parameters());
        optimizer.step(graph, &gradients);
        Roles::new()
            .axis("batch", batch)
            .input("observation", observation)
            .input("target", target)
            .output("loss", loss)
    })
}

fn errors(weight: f32) -> Vec<f32> {
    VALUES
        .iter()
        .zip(TARGETS)
        .map(|(value, target)| weight * value - target)
        .collect()
}

fn loss_at(weight: f32) -> f32 {
    errors(weight)
        .iter()
        .map(|error| error * error)
        .sum::<f32>()
        / VALUES.len() as f32
}

fn gradient_at(weight: f32) -> f32 {
    2.0 * errors(weight)
        .iter()
        .zip(VALUES)
        .map(|(error, value)| error * value)
        .sum::<f32>()
        / VALUES.len() as f32
}

fn written(values: [f32; 4], targets: [f32; 4]) -> Inputs {
    Inputs::new()
        .write("observation", values)
        .write("target", targets)
}

fn close(actual: f32, expected: f32, tolerance: f32) -> bool {
    (actual - expected).abs() <= tolerance * expected.abs().max(1.0)
}

#[test]
fn a_run_updates_the_weights_its_program_holds() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = scalar().attach(runtime);
    model.run(runtime, &written(VALUES, TARGETS));
    let loss = model.read(runtime, "loss")[0];
    assert!(
        close(loss, loss_at(START), 1e-5),
        "the run reports {loss} where the squared error of {START} reads {}",
        loss_at(START),
    );
    let weight = model.read(runtime, "weight")[0];
    assert!(
        close(weight, START - RATE * gradient_at(START), 1e-4),
        "the run descends by {RATE} along the gradient of {}",
        gradient_at(START),
    );
}

#[test]
fn every_run_reads_the_loss_of_its_own_inputs() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = scalar().attach(runtime);
    let values = [1.0, -0.5, 0.25, -2.0];
    let targets = [-1.0, 2.0, 0.0, 1.0];
    model.run(runtime, &written(VALUES, TARGETS));
    let first = model.read(runtime, "loss")[0];
    assert!(close(first, loss_at(START), 1e-5));
    let moved = START - RATE * gradient_at(START);
    model.run(runtime, &written(values, targets));
    let second = model.read(runtime, "loss")[0];
    let expected = values
        .iter()
        .zip(targets)
        .map(|(value, target)| {
            let error = moved * value - target;
            error * error
        })
        .sum::<f32>()
        / values.len() as f32;
    assert!(
        close(second, expected, 1e-5),
        "the second run reads {second} where the weight the first left reads {expected}",
    );
    let gradient = 2.0
        * values
            .iter()
            .zip(targets)
            .map(|(value, target)| (moved * value - target) * value)
            .sum::<f32>()
        / values.len() as f32;
    let weight = model.read(runtime, "weight")[0];
    assert!(close(weight, moved - RATE * gradient, 1e-4));
}

#[test]
fn repeated_runs_converge_on_the_reference() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = dynamic().attach(runtime);
    model.bind(runtime, &Extents::new().axis("batch", 1));
    let mut first = None;
    let mut last = 0.0;
    for step in 0..300 {
        let row = (step % 16) as f32 / 8.0 - 1.0;
        model.run(
            runtime,
            &Inputs::new()
                .write("observation", [row, 1.0])
                .write("target", [2.0 * row + 1.0]),
        );
        if step == 0 || step == 299 {
            last = model.read(runtime, "loss")[0];
            first.get_or_insert(last);
        }
    }
    let first = first.expect("the first run reads a loss");
    assert!(
        last < first / 100.0,
        "the run reads {last} at the last step where it read {first} at the first",
    );
}
