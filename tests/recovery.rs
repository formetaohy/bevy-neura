mod common;

use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy_neura::{Inputs, Model, ModelPlan, NeuraRebuilt, NeuraRuntime, Roles};
use neura::{Element, Init, Linear, MemoryRequest, Shape};

#[derive(Resource, Default)]
struct Answer(f32);

#[derive(Resource, Default)]
struct Rebuilds(usize);

fn another_device() -> (wgpu::Device, wgpu::Adapter) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: common::settings().backends.unwrap_or_default(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = bevy::tasks::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..default()
    }))
    .expect("a second adapter of this App");
    let (device, queue) = bevy::tasks::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("another device of the App"),
        ..default()
    }))
    .expect("a second device of this App");
    drop(queue);
    (device, adapter)
}

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

fn rebuild_models(
    runtime: Res<NeuraRuntime>,
    mut rebuilt: MessageReader<NeuraRebuilt>,
    mut rebuilds: ResMut<Rebuilds>,
    mut models: Query<&mut Model>,
) {
    for _ in rebuilt.read() {
        rebuilds.0 += 1;
        for mut model in &mut models {
            *model = dense().attach(&runtime);
        }
    }
}

fn drive(runtime: Res<NeuraRuntime>, models: Query<&Model>, mut answer: ResMut<Answer>) {
    for model in &models {
        model.run(&runtime, &Inputs::new().write("observation", [1.5]));
        answer.0 = model.read(&runtime, "answer")[0];
    }
}

#[test]
fn a_replaced_render_device_rebuilds_the_runtime() {
    let mut app = common::app(MemoryRequest::default());
    app.init_resource::<Answer>()
        .init_resource::<Rebuilds>()
        .add_systems(Startup, spawn_model)
        .add_systems(Update, (rebuild_models, drive).chain());
    app.update();

    let (device, keep_alive) = another_device();
    let another_host = !app.world().resource::<NeuraRuntime>().answers_to(&device);
    app.world_mut().insert_resource(RenderDevice::from(device));
    app.update();

    let runtime = app.world().resource::<NeuraRuntime>();
    let rendered = app.world().resource::<RenderDevice>();
    assert!(
        runtime.answers_to(rendered.wgpu_device()),
        "the runtime does not answer to the device the App renders on",
    );
    assert_eq!(
        app.world().resource::<Rebuilds>().0,
        usize::from(another_host),
        "the App was not told of a rebuilt runtime",
    );
    for _ in 0..3 {
        app.update();
        let answer = app.world().resource::<Answer>().0;
        assert!(
            (answer - 3.0).abs() < 1e-6,
            "a layer of weight 2 answers {answer} where twice 1.5 reads 3",
        );
    }
    drop(app);
    drop(keep_alive);
}
