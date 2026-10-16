use bevy::prelude::Resource;
use neura::{HostIdentity, Runtime};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct NeuraRuntime {
    runtime: Arc<Runtime>,
    identity: HostIdentity,
}

impl NeuraRuntime {
    pub(crate) fn borrowed(runtime: Runtime, identity: HostIdentity) -> Self {
        Self {
            runtime: Arc::new(runtime),
            identity,
        }
    }

    pub fn answers_to(&self, device: &wgpu::Device) -> bool {
        crate::host::identity(device) == self.identity
    }
}

impl Deref for NeuraRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Runtime {
        &self.runtime
    }
}
