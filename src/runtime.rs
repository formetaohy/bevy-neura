use bevy::prelude::Resource;
use neura::{Runtime, RuntimeRequest};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct NeuraRuntime(Arc<Runtime>);

impl NeuraRuntime {
    pub fn open(request: RuntimeRequest) -> Self {
        match Runtime::open(request) {
            Ok(runtime) => Self(Arc::new(runtime)),
            Err(unavailable) => panic!("the Neura device does not open: {unavailable}"),
        }
    }
}

impl Deref for NeuraRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Runtime {
        &self.0
    }
}
