use bevy::prelude::*;
use bevy_neura::{
    Extents, ModelId, ModelPlan, NeuraDevice, NeuraHandle, NeuraPlugin, NeuraReport, RequestId,
    Roles, Sample,
};
use neura::{AdapterId, AdapterPolicy, Element, GpuRequest, Init, Linear, RuntimeRequest, Shape};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    app
}

fn plan() -> ModelPlan {
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
        let prediction = dense.forward(graph, observation);
        graph.retain(prediction);
        Roles::new()
            .input("observation", observation)
            .output("prediction", prediction)
    })
}

fn answered(reports: &[NeuraReport], request: RequestId) -> bool {
    reports.iter().any(
        |report| matches!(report, NeuraReport::Inferred { request: answered, .. } if *answered == request),
    )
}

fn attached(reports: &[NeuraReport], model: ModelId) -> bool {
    reports.iter().any(
        |report| matches!(report, NeuraReport::Attached { model: attached, .. } if *attached == model),
    )
}

#[test]
fn the_device_of_the_app_reaches_a_resource() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    let device = app.world().resource::<NeuraDevice>();
    assert!(!device.adapter.name.is_empty());
    assert!(device.heap_bytes > 0);
    assert!(device.readback_bytes > 0);
}

#[test]
fn the_app_reports_the_answers_of_the_device() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::default());
    app.update();
    let (model, request) = {
        let handle = app.world().resource::<NeuraHandle>();
        let model = handle.attach(plan());
        let request = handle.infer(
            model,
            Extents::new(),
            Sample::new().write("observation", [1.0, 2.0, 3.0, 4.0]),
        );
        (model, request)
    };
    let mut reports = Vec::new();
    for _ in 0..600 {
        app.update();
        reports.extend(
            app.world_mut()
                .resource_mut::<Messages<NeuraReport>>()
                .drain(),
        );
        if answered(&reports, request) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        attached(&reports, model),
        "the attachment of the model reaches a message",
    );
    assert!(
        answered(&reports, request),
        "the answer of the device reaches a message",
    );
}

#[test]
#[should_panic(expected = "the Neura device does not open")]
fn a_device_that_no_adapter_answers_fails_the_app() {
    let mut app = App::new();
    app.add_plugins(NeuraPlugin::new(RuntimeRequest {
        gpu: GpuRequest {
            adapter: AdapterPolicy::Identity(AdapterId::Numeric {
                vendor: 0xdead,
                device: 0xbeef,
            }),
            ..GpuRequest::default()
        },
        ..RuntimeRequest::default()
    }));
    app.update();
}

#[test]
#[should_panic(expected = "the Neura device thread stopped")]
fn a_worker_that_stops_fails_the_caller_that_waits() {
    let app = app();
    let handle = app.world().resource::<NeuraHandle>();
    let model = handle.attach(plan());
    let path = std::env::temp_dir().join(format!(
        "bevy-neura-broken-{}.safetensors",
        std::process::id()
    ));
    std::fs::write(&path, "no checkpoint reaches a device")
        .expect("a file lands in the temp directory");
    let request = handle.restore(model, &path);
    let _ = handle.wait(request);
}

#[test]
#[should_panic(expected = "the reports it owes this app never arrive")]
fn a_worker_that_stops_fails_the_app_that_drains() {
    let mut app = app();
    {
        let handle = app.world().resource::<NeuraHandle>();
        let model = handle.attach(plan());
        let path = std::env::temp_dir().join(format!(
            "bevy-neura-broken-{}.safetensors",
            std::process::id()
        ));
        std::fs::write(&path, "no checkpoint reaches a device")
            .expect("a file lands in the temp directory");
        let _ = handle.restore(model, &path);
    }
    for _ in 0..600 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
