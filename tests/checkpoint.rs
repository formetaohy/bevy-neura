use bevy::prelude::*;
use bevy_neura::{
    Extents, Loss, ModelId, ModelPlan, NeuraHandle, NeuraPlugin, NeuraReport, Roles, Sample,
};
use neura::{AdamW, Element, Init, Linear, Shape, mse_loss};

const OBSERVATIONS: [f32; 4] = [0.25, -0.5, 1.5, 2.0];
const TARGETS: [f32; 4] = [-1.0, 0.5, 2.0, -1.5];

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    app
}

fn learner() -> ModelPlan {
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
            .loss(loss)
    })
}

fn actor() -> ModelPlan {
    ModelPlan::build(|graph| {
        let dense = Linear::new(graph, "dense", 2, 2, Init::Zero, Element::Single);
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let prediction = dense.forward(graph, observation);
        graph.retain(prediction);
        Roles::new()
            .input("observation", observation)
            .output("prediction", prediction)
    })
}

fn predict(handle: &NeuraHandle, model: ModelId) -> Vec<f32> {
    let request = handle.infer(
        model,
        Extents::new(),
        Sample::new().write("observation", OBSERVATIONS),
    );
    let NeuraReport::Inferred { mut values, .. } = handle.wait(request) else {
        panic!("an inference answers the inference that reached the device")
    };
    values.pop().expect("one output answered the inference").1
}

fn distance(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f32::max)
}

#[test]
fn a_checkpoint_returns_the_weights_of_the_learner_it_holds() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let learner = handle.attach(learner());
    let actor = handle.attach_shared(learner, actor());
    let before = predict(handle, actor);
    let path = std::env::temp_dir().join(format!("bevy-neura-{}.safetensors", std::process::id()));
    let saved = handle.save(learner, &path);
    let NeuraReport::Saved { bytes, .. } = handle.wait(saved) else {
        panic!("a save answers the save that reached the device")
    };
    assert!(bytes > 0);
    let samples = (0..8)
        .map(|_| {
            Sample::new()
                .write("observation", OBSERVATIONS)
                .write("target", TARGETS)
        })
        .collect::<Vec<Sample>>();
    let trained = handle.train(learner, Extents::new(), samples, Loss::None);
    let NeuraReport::Trained { steps, losses, .. } = handle.wait(trained) else {
        panic!("a training answers the training that reached the device")
    };
    assert_eq!(steps, 8);
    assert!(losses.is_empty());
    let moved = predict(handle, actor);
    assert!(
        distance(&before, &moved) > 1e-4,
        "eight descents move the prediction the actor shares",
    );
    let restored = handle.restore(learner, &path);
    let NeuraReport::Restored { .. } = handle.wait(restored) else {
        panic!("a restore answers the restore that reached the device")
    };
    let after = predict(handle, actor);
    assert!(
        distance(&before, &after) < 1e-6,
        "the checkpoint returns the prediction the shared weights held before the descents",
    );
    std::fs::remove_file(&path).expect("the checkpoint of the test leaves no file behind");
}
