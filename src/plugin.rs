use crate::queue;
use crate::rebuild::{NeuraRebuilt, borrow_runtime};
use bevy::prelude::{App, First, Plugin, PreStartup, Resource};
use neura::RuntimeRequest;

#[derive(Resource)]
pub(crate) struct NeuraRequest(RuntimeRequest);

impl NeuraRequest {
    pub(crate) fn of(request: &RuntimeRequest) -> Self {
        Self(RuntimeRequest {
            gpu: request.gpu.clone(),
            memory: request.memory.clone(),
        })
    }

    pub(crate) fn requested(&self) -> RuntimeRequest {
        RuntimeRequest {
            gpu: self.0.gpu.clone(),
            memory: self.0.memory.clone(),
        }
    }
}

#[derive(Default)]
pub struct NeuraPlugin {
    request: RuntimeRequest,
}

impl NeuraPlugin {
    pub fn of(request: RuntimeRequest) -> Self {
        Self { request }
    }
}

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NeuraRequest::of(&self.request));
        app.add_message::<NeuraRebuilt>();
        app.add_systems(PreStartup, borrow_runtime);
        app.add_systems(First, borrow_runtime);
    }

    fn finish(&self, app: &mut App) {
        queue::hold(app);
    }
}
