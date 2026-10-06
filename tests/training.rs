use bevy::prelude::*;
use bevy_neura::{
    Extents, Loss, ModelId, ModelPlan, NeuraHandle, NeuraPlugin, NeuraReport, Roles, Sample,
};
use neura::{AdamW, Element, Init, Linear, Sgd, Shape, mse_loss};

const OBSERVATIONS: [f32; 4] = [0.5, 1.5, -1.0, 2.0];
const TARGETS: [f32; 4] = [1.0, 1.0, -1.0, 0.5];
const RATE: f32 = 0.1;
const START: f32 = 0.5;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    app
}

fn linear() -> ModelPlan {
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
            .loss(loss)
    })
}

fn regression() -> ModelPlan {
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
            .loss(loss)
    })
}

fn errors(weight: f32) -> Vec<f32> {
    OBSERVATIONS
        .iter()
        .zip(TARGETS)
        .map(|(observation, target)| weight * observation - target)
        .collect()
}

fn loss_at(weight: f32) -> f32 {
    errors(weight)
        .iter()
        .map(|error| error * error)
        .sum::<f32>()
        / OBSERVATIONS.len() as f32
}

fn gradient_at(weight: f32) -> f32 {
    2.0 * errors(weight)
        .iter()
        .zip(OBSERVATIONS)
        .map(|(error, observation)| error * observation)
        .sum::<f32>()
        / OBSERVATIONS.len() as f32
}

fn sample(observations: [f32; 4], targets: [f32; 4]) -> Sample {
    Sample::new()
        .write("observation", observations)
        .write("target", targets)
}

fn close(actual: f32, expected: f32, tolerance: f32) -> bool {
    (actual - expected).abs() <= tolerance * expected.abs().max(1.0)
}

fn weight_of(handle: &NeuraHandle, model: ModelId) -> f32 {
    let request = handle.read(model, Extents::new(), vec!["weight"]);
    let NeuraReport::Read { mut values, .. } = handle.wait(request) else {
        panic!("a read answers the read that reached the device")
    };
    values
        .pop()
        .expect("one output answered the read")
        .1
        .first()
        .copied()
        .expect("a weight holds one number")
}

#[test]
fn a_step_reports_the_loss_it_ran_and_moves_the_weight() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(linear());
    let request = handle.train(
        model,
        Extents::new(),
        vec![sample(OBSERVATIONS, TARGETS)],
        Loss::Every,
    );
    let NeuraReport::Trained { losses, .. } = handle.wait(request) else {
        panic!("a training answers the training that reached the device")
    };
    assert_eq!(losses.len(), 1);
    assert!(
        close(losses[0], loss_at(START), 1e-5),
        "the step reports {} where the squared error of {START} reads {}",
        losses[0],
        loss_at(START),
    );
    assert!(
        close(
            weight_of(handle, model),
            START - RATE * gradient_at(START),
            1e-4
        ),
        "the step descends by {RATE} along the gradient of {}",
        gradient_at(START),
    );
}

#[test]
fn every_step_reads_the_loss_of_its_own_sample() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(linear());
    let observations = [1.0, -0.5, 0.25, -2.0];
    let targets = [-1.0, 2.0, 0.0, 1.0];
    let request = handle.train(
        model,
        Extents::new(),
        vec![
            sample(OBSERVATIONS, TARGETS),
            Sample::new()
                .write("observation", observations)
                .write("target", targets),
        ],
        Loss::Every,
    );
    let NeuraReport::Trained { losses, .. } = handle.wait(request) else {
        panic!("a training answers the training that reached the device")
    };
    assert_eq!(losses.len(), 2);
    assert!(close(losses[0], loss_at(START), 1e-5));
    let moved = START - RATE * gradient_at(START);
    let second = observations
        .iter()
        .zip(targets)
        .map(|(observation, target)| {
            let error = moved * observation - target;
            error * error
        })
        .sum::<f32>()
        / observations.len() as f32;
    assert!(
        close(losses[1], second, 1e-5),
        "the second step reports {} where the weight the first left reads {second}",
        losses[1],
    );
    let gradient = 2.0
        * observations
            .iter()
            .zip(targets)
            .map(|(observation, target)| (moved * observation - target) * observation)
            .sum::<f32>()
        / observations.len() as f32;
    assert!(close(
        weight_of(handle, model),
        moved - RATE * gradient,
        1e-4
    ));
}

#[test]
#[should_panic(expected = "runs the descents of the graph that trains it")]
fn an_inference_of_a_training_graph_reaches_no_device() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(linear());
    let _ = handle.infer(model, Extents::new(), sample(OBSERVATIONS, TARGETS));
}

#[test]
fn a_line_learns_from_one_row_at_a_time() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(regression());
    let samples = (0..300)
        .map(|step| {
            let row = (step % 16) as f32 / 8.0 - 1.0;
            Sample::new()
                .write("observation", [row, 1.0])
                .write("target", [2.0 * row + 1.0])
        })
        .collect::<Vec<Sample>>();
    let request = handle.train(model, Extents::new().axis("batch", 1), samples, Loss::Every);
    let NeuraReport::Trained { losses, .. } = handle.wait(request) else {
        panic!("a training answers the training that reached the device")
    };
    assert_eq!(losses.len(), 300);
    assert!(
        losses[299] < losses[0] / 100.0,
        "the line reads {} at the last step where it read {} at the first",
        losses[299],
        losses[0],
    );
}
