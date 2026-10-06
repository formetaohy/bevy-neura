use crate::runtime::NeuraRuntime;
use bevy::prelude::{App, Plugin};
use neura::RuntimeRequest;

pub struct NeuraPlugin(NeuraRuntime);

impl NeuraPlugin {
    pub fn new(request: RuntimeRequest) -> Self {
        Self(NeuraRuntime::open(request))
    }
}

impl Default for NeuraPlugin {
    fn default() -> Self {
        Self::new(RuntimeRequest::default())
    }
}

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone());
    }
}
