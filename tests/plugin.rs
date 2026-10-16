mod common;

use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy_neura::{Inputs, Model, ModelPlan, NeuraPlugin, NeuraRuntime, Roles};
use neura::{Element, GpuRequest, Init, Linear, MemoryRequest, RuntimeRequest, Shape};

#[derive(Resource, Default)]
struct Answer(f32);

fn dense() -> ModelPlan {
    ModelPlan::build(|graph| {
        let dense = Linear::new(graph, "dense", 1, 1, Init::Constant(2.0), Element::Single);
        let observation = graph.input(Shape::matrix(1, 1), Element::Single);
        let answer = dense.forward(graph, observation);
        Roles::new()
            .input("observation", observation)
            .output("answer", answer)
    })
}

fn spawn_model(runtime: Res<NeuraRuntime>, mut commands: Commands) {
    commands.spawn(dense().attach(&runtime));
}

fn drive(runtime: Res<NeuraRuntime>, models: Query<&Model>, mut answer: ResMut<Answer>) {
    for model in &models {
        model.run(&runtime, &Inputs::new().write("observation", [1.5]));
        answer.0 = model.read(&runtime, "answer")[0];
    }
}

#[test]
fn the_runtime_borrows_the_device_of_the_app() {
    let mut app = common::app(MemoryRequest::default());
    app.update();
    let runtime = app.world().resource::<NeuraRuntime>();
    let rendered = app.world().resource::<RenderAdapterInfo>();
    assert_eq!(
        runtime.context().adapter_info().name,
        rendered.name,
        "the runtime holds the compute device of {} where the App renders on {}",
        runtime.context().adapter_info().name,
        rendered.name,
    );
    assert!(runtime.heap_bytes() > 0);
    assert!(runtime.readback_capacity() > 0);
    assert!(runtime.readback_slots() > 0);
}

#[test]
fn a_request_of_the_app_reaches_the_runtime() {
    let artifacts = std::env::temp_dir().join("bevy-neura-request");
    let mut app = common::assemble(
        RuntimeRequest {
            gpu: GpuRequest {
                artifacts: Some(artifacts.clone()),
                artifact_bytes: 1 << 20,
                ..GpuRequest::default()
            },
            memory: MemoryRequest {
                heap_bytes: 1 << 24,
                readback_bytes: 1 << 20,
                readback_slots: 3,
                ..MemoryRequest::default()
            },
        },
        false,
    );
    app.update();
    let runtime = app.world().resource::<NeuraRuntime>();
    assert_eq!(runtime.heap_bytes(), 1 << 24);
    assert_eq!(runtime.readback_capacity(), 1 << 20);
    assert_eq!(runtime.readback_slots(), 3);
    let cache = runtime.context().artifact_cache();
    assert_eq!(cache.root(), Some(artifacts.as_path()));
    assert_eq!(cache.budget(), 1 << 20);
}

#[test]
fn a_system_drives_a_model_from_the_schedule() {
    let mut app = common::app(MemoryRequest::default());
    app.init_resource::<Answer>()
        .add_systems(Startup, spawn_model)
        .add_systems(Update, drive);
    for _ in 0..3 {
        app.update();
        let answer = app.world().resource::<Answer>().0;
        assert!(
            (answer - 3.0).abs() < 1e-6,
            "a layer of weight 2 answers {answer} where twice 1.5 reads 3",
        );
    }
}

#[test]
#[should_panic(expected = "the App carries no render device")]
fn an_app_without_a_render_device_reaches_no_runtime() {
    App::new().add_plugins(NeuraPlugin::default()).update();
}

#[test]
fn a_pipelined_app_drives_a_model_from_the_schedule() {
    let mut app = common::assemble(RuntimeRequest::default(), true);
    app.init_resource::<Answer>()
        .add_systems(Startup, spawn_model)
        .add_systems(Update, drive);
    for _ in 0..8 {
        app.update();
        let answer = app.world().resource::<Answer>().0;
        assert!(
            (answer - 3.0).abs() < 1e-6,
            "a layer of weight 2 answers {answer} where twice 1.5 reads 3",
        );
    }
}
