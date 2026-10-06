use bevy::prelude::*;
use bevy_neura::{Inputs, Model, ModelPlan, NeuraPlugin, NeuraRuntime, Roles};
use neura::{AdapterId, AdapterPolicy, Element, GpuRequest, Init, Linear, RuntimeRequest, Shape};

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
fn the_device_of_the_app_reaches_a_resource() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    let runtime = app.world().resource::<NeuraRuntime>();
    assert!(!runtime.context().adapter_info().name.is_empty());
    assert!(runtime.heap_bytes() > 0);
    assert!(runtime.readback_capacity() > 0);
    assert!(runtime.readback_slots() > 0);
}

#[test]
fn a_system_drives_a_model_from_the_schedule() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default())
        .init_resource::<Answer>()
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
#[should_panic(expected = "the Neura device does not open")]
fn a_device_that_no_adapter_answers_fails_the_plugin() {
    let _ = NeuraPlugin::new(RuntimeRequest {
        gpu: GpuRequest {
            adapter: AdapterPolicy::Identity(AdapterId::Numeric {
                vendor: 0xdead,
                device: 0xbeef,
            }),
            ..GpuRequest::default()
        },
        ..RuntimeRequest::default()
    });
}
