use bevy::prelude::*;
use bevy_neura::{Extents, ModelPlan, NeuraHandle, NeuraPlugin, NeuraReport, Roles, Sample};
use neura::{Element, Init, Linear, Shape};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    app
}

fn shifted_twice() -> ModelPlan {
    ModelPlan::build(|graph| {
        let batch = graph.free(8);
        let observation = graph.input(Shape::matrix(8, 2).freed(&[(2, batch)]), Element::Single);
        let prediction = graph.add(
            graph.mul(observation, graph.fill(Shape::scalar(), 2.0)),
            graph.fill(Shape::scalar(), 1.0),
        );
        graph.retain(prediction);
        Roles::new()
            .axis("batch", batch)
            .input("observation", observation)
            .output("prediction", prediction)
    })
}

fn infer(handle: &NeuraHandle, model: bevy_neura::ModelId, batch: u32, data: Vec<f32>) -> Vec<f32> {
    let request = handle.infer(
        model,
        Extents::new().axis("batch", batch),
        Sample::new().write("observation", data),
    );
    let NeuraReport::Inferred { mut values, .. } = handle.wait(request) else {
        panic!("an inference answers the inference that reached the device")
    };
    assert_eq!(values.len(), 1);
    values.pop().expect("one output answered the inference").1
}

fn doubled(data: &[f32]) -> Vec<f32> {
    data.iter().map(|value| value * 2.0 + 1.0).collect()
}

#[test]
fn inference_follows_the_live_extent() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    for batch in [8u32, 3, 1] {
        let data = (0..batch * 2)
            .map(|value| value as f32 * 0.25 - 1.0)
            .collect::<Vec<f32>>();
        assert_eq!(infer(handle, model, batch, data.clone()), doubled(&data));
    }
}

#[test]
fn a_live_extent_of_zero_runs() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    assert!(infer(handle, model, 0, Vec::new()).is_empty());
}

#[test]
fn a_read_of_an_output_answers_the_storage_of_the_last_run() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    let data = vec![1.0, 2.0, 3.0, 4.0];
    assert_eq!(infer(handle, model, 2, data.clone()), doubled(&data));
    let request = handle.read(model, Extents::new().axis("batch", 2), vec!["prediction"]);
    let NeuraReport::Read { values, .. } = handle.wait(request) else {
        panic!("a read answers the read that reached the device")
    };
    assert_eq!(values, vec![("prediction", doubled(&data))]);
}

#[test]
fn two_extents_bind_by_their_roles() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(ModelPlan::build(|graph| {
        let rows = graph.free(3);
        let columns = graph.free(2);
        let matrix = graph.input(
            Shape::matrix(3, 2).freed(&[(2, rows), (3, columns)]),
            Element::Single,
        );
        let scaled = graph.mul(matrix, graph.fill(Shape::scalar(), 3.0));
        graph.retain(scaled);
        Roles::new()
            .axis("columns", columns)
            .axis("rows", rows)
            .input("matrix", matrix)
            .output("scaled", scaled)
    }));
    let request = handle.infer(
        model,
        Extents::new().axis("rows", 3).axis("columns", 1),
        Sample::new().write("matrix", [1.0, 2.0, 3.0]),
    );
    let NeuraReport::Inferred { values, .. } = handle.wait(request) else {
        panic!("an inference answers the inference that reached the device")
    };
    assert_eq!(values, vec![("scaled", vec![3.0, 6.0, 9.0])]);
}

#[test]
fn the_geometry_of_an_attachment_names_its_bounds() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    let geometry = handle.wait_attached(model);
    assert_eq!(geometry.bounds, vec![("batch", 8)]);
    assert!(geometry.tasks > 0);
    assert!(geometry.workgroups > 0);
    assert!(geometry.tensors > 0);
    assert!(geometry.dynamic);
    assert!(!geometry.updates);
}

#[test]
fn a_tune_answers_the_profile_it_chooses() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(ModelPlan::build(|graph| {
        let dense = Linear::new(graph, "dense", 2, 2, Init::Zero, Element::Single);
        let observation = graph.input(Shape::matrix(4, 2), Element::Single);
        let prediction = dense.forward(graph, observation);
        graph.retain(prediction);
        Roles::new()
            .input("observation", observation)
            .output("prediction", prediction)
    }));
    let request = handle.tune(model, Extents::new());
    let NeuraReport::Tuned {
        profile, seconds, ..
    } = handle.wait(request)
    else {
        panic!("a tune answers the tune that reached the device")
    };
    assert!(seconds.is_finite());
    assert!(
        profile.workgroup() > 0,
        "a tuned program walks the workgroup the device offers",
    );
    let request = handle.infer(
        model,
        Extents::new(),
        Sample::new().write("observation", vec![0.0; 8]),
    );
    let NeuraReport::Inferred { values, .. } = handle.wait(request) else {
        panic!("an inference answers the inference that reached the device")
    };
    assert_eq!(values.len(), 1);
}

#[test]
#[should_panic(expected = "is no attached model")]
fn a_detached_model_reaches_no_further_command() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    handle.wait_attached(model);
    handle.detach(model);
    let _ = handle.tune(model, Extents::new());
}

#[test]
#[should_panic(expected = "a write of 6 numbers reaches the input observation of 4 numbers")]
fn a_write_of_the_wrong_size_reaches_no_device() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    let _ = handle.infer(
        model,
        Extents::new().axis("batch", 2),
        Sample::new().write("observation", vec![0.0; 6]),
    );
}

#[test]
#[should_panic(expected = "a live batch of 9 outruns the bound of 8")]
fn a_live_extent_beyond_the_bound_reaches_no_device() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    let _ = handle.infer(
        model,
        Extents::new().axis("batch", 9),
        Sample::new().write("observation", vec![0.0; 18]),
    );
}

#[test]
#[should_panic(expected = "the live length of rows binds no host extent of this model")]
fn a_live_extent_of_another_name_reaches_no_device() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(shifted_twice());
    let _ = handle.tune(model, Extents::new().axis("rows", 2));
}
