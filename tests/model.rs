use bevy::prelude::*;
use bevy_neura::{Extents, Inputs, Model, ModelPlan, NeuraPlugin, NeuraRuntime, Roles};
use neura::{Element, Init, Linear, MemoryRequest, RuntimeRequest, Shape};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app
}

fn affine() -> ModelPlan {
    ModelPlan::build(|graph| {
        let batch = graph.free(8);
        let observation = graph.input(Shape::matrix(8, 2).freed(&[(2, batch)]), Element::Single);
        let prediction = graph.add(
            graph.mul(observation, graph.fill(Shape::scalar(), 2.0)),
            graph.fill(Shape::scalar(), 1.0),
        );
        Roles::new()
            .axis("batch", batch)
            .input("observation", observation)
            .output("prediction", prediction)
    })
}

fn affine_of(data: &[f32]) -> Vec<f32> {
    data.iter().map(|value| value * 2.0 + 1.0).collect()
}

fn run_affine(runtime: &NeuraRuntime, model: &Model, batch: u32, data: Vec<f32>) -> Vec<f32> {
    model.bind(runtime, &Extents::new().axis("batch", batch));
    model.run(runtime, &Inputs::new().write("observation", data));
    model.read(runtime, "prediction")
}

#[test]
fn a_run_follows_the_live_extent() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    for batch in [8u32, 3, 1] {
        let data = (0..batch * 2)
            .map(|value| value as f32 * 0.25 - 1.0)
            .collect::<Vec<f32>>();
        assert_eq!(
            run_affine(runtime, &model, batch, data.clone()),
            affine_of(&data)
        );
    }
}

#[test]
fn a_live_extent_of_zero_runs() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    assert!(run_affine(runtime, &model, 0, Vec::new()).is_empty());
}

#[test]
fn two_extents_bind_by_their_roles() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = ModelPlan::build(|graph| {
        let rows = graph.free(3);
        let columns = graph.free(2);
        let matrix = graph.input(
            Shape::matrix(3, 2).freed(&[(2, rows), (3, columns)]),
            Element::Single,
        );
        let scaled = graph.mul(matrix, graph.fill(Shape::scalar(), 3.0));
        Roles::new()
            .axis("columns", columns)
            .axis("rows", rows)
            .input("matrix", matrix)
            .output("scaled", scaled)
    })
    .attach(runtime);
    model.bind(runtime, &Extents::new().axis("rows", 3).axis("columns", 1));
    model.run(runtime, &Inputs::new().write("matrix", [1.0, 2.0, 3.0]));
    assert_eq!(model.read(runtime, "scaled"), vec![3.0, 6.0, 9.0]);
}

#[test]
fn every_output_answers_the_last_run() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let sum = graph.sum_axis(observation, 3);
        let scaled = graph.mul(observation, graph.fill(Shape::scalar(), 0.5));
        Roles::new()
            .input("observation", observation)
            .output("sum", sum)
            .output("scaled", scaled)
    })
    .attach(runtime);
    model.run(
        runtime,
        &Inputs::new().write("observation", [1.0, 3.0, 5.0, 7.0]),
    );
    assert_eq!(
        model.read_all(runtime),
        vec![
            ("sum", vec![4.0, 12.0]),
            ("scaled", vec![0.5, 1.5, 2.5, 3.5]),
        ],
    );
}

#[test]
fn a_pull_of_every_model_collects_afterwards() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let models = [affine().attach(runtime), affine().attach(runtime)];
    let mut pending = Vec::new();
    for (agent, model) in models.iter().enumerate() {
        model.bind(runtime, &Extents::new().axis("batch", 1));
        model.run(
            runtime,
            &Inputs::new().write("observation", [agent as f32, agent as f32]),
        );
        pending.push(model.pull(runtime, &["prediction"]));
    }
    let read = pending
        .into_iter()
        .map(|readout| readout.collect())
        .collect::<Vec<_>>();
    assert_eq!(read[0], vec![vec![1.0, 1.0]]);
    assert_eq!(read[1], vec![vec![3.0, 3.0]]);
}

#[test]
fn a_value_of_a_role_serves_the_raw_runtime() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new().axis("batch", 1));
    runtime.write(model.program(), model.value("observation"), &[1.0, 2.0]);
    runtime.run(model.program());
    assert_eq!(
        runtime.read(model.program(), model.value("prediction")),
        vec![3.0, 5.0],
    );
}

#[test]
fn the_geometry_of_a_model_names_its_bounds() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    let geometry = model.geometry();
    assert_eq!(geometry.bounds, vec![("batch", 8)]);
    assert!(geometry.tasks > 0);
    assert!(geometry.workgroups > 0);
    assert!(geometry.tensors > 0);
    assert!(geometry.dynamic);
    assert!(!geometry.updates);
    assert_eq!(model.program().task_count(), geometry.tasks);
    assert_eq!(model.weights().tensors(), 0);
}

#[test]
fn a_tuned_model_runs_the_values_it_compiled() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let scaled = graph.mul(observation, graph.fill(Shape::scalar(), 3.0));
        Roles::new()
            .input("observation", observation)
            .output("scaled", scaled)
    })
    .attach_tuned(runtime);
    model.run(
        runtime,
        &Inputs::new().write("observation", [1.0, -1.0, 2.0, 0.5]),
    );
    assert_eq!(model.read(runtime, "scaled"), vec![3.0, -3.0, 6.0, 1.5]);
}

#[test]
fn a_dropped_model_returns_its_storage_to_the_device_heap() {
    let roomy = app();
    let probe = dense().attach(roomy.world().resource::<NeuraRuntime>());
    let geometry = probe.geometry();
    let footprint = geometry.weights + geometry.tensors;
    drop(probe);

    let mut tight = App::new();
    tight.add_plugins(NeuraPlugin::new(RuntimeRequest {
        memory: MemoryRequest {
            heap_bytes: footprint + footprint / 2,
            ..MemoryRequest::default()
        },
        ..RuntimeRequest::default()
    }));
    let runtime = tight.world().resource::<NeuraRuntime>();
    assert!(
        runtime.heap_bytes() < footprint * 2,
        "the heap of this test holds one model of {footprint} bytes and not two",
    );
    for _ in 0..4 {
        let model = dense().attach(runtime);
        model.run(
            runtime,
            &Inputs::new().write("observation", vec![1.0; 32 * 64]),
        );
        assert_eq!(model.read(runtime, "answer").len(), 32 * 64);
        drop(model);
    }
}

fn dense() -> ModelPlan {
    ModelPlan::build(|graph| {
        let dense = Linear::new(
            graph,
            "dense",
            64,
            64,
            Init::Uniform {
                low: -0.1,
                high: 0.1,
            },
            Element::Single,
        );
        let observation = graph.input(Shape::matrix(32, 64), Element::Single);
        let answer = dense.forward(graph, observation);
        Roles::new()
            .input("observation", observation)
            .output("answer", answer)
    })
}

#[test]
#[should_panic(expected = "no run has named one yet")]
fn a_run_before_the_binding_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.run(runtime, &Inputs::new().write("observation", [0.0, 1.0]));
}

#[test]
#[should_panic(expected = "writing 6 numbers into a tensor of 4 numbers")]
fn a_write_of_another_size_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new().axis("batch", 2));
    model.run(runtime, &Inputs::new().write("observation", vec![0.0; 6]));
}

#[test]
#[should_panic(expected = "outruns the bound of 8 the graph declares")]
fn a_live_extent_beyond_the_bound_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new().axis("batch", 9));
}

#[test]
#[should_panic(expected = "the live length of rows binds no host extent of this model")]
fn a_live_extent_of_another_name_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new().axis("rows", 2));
}

#[test]
#[should_panic(expected = "a model of 1 host extents binds 0 live lengths")]
fn a_binding_that_names_no_extent_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new());
}

#[test]
#[should_panic(expected = "the model holds no input under the role target")]
fn a_write_to_an_undeclared_role_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    model.bind(runtime, &Extents::new().axis("batch", 1));
    model.run(runtime, &Inputs::new().write("target", [1.0, 2.0]));
}

#[test]
#[should_panic(expected = "the model holds no output under the role loss")]
fn a_read_of_an_undeclared_role_reaches_no_device() {
    let app = app();
    let runtime = app.world().resource::<NeuraRuntime>();
    let model = affine().attach(runtime);
    let _ = model.read(runtime, "loss");
}
