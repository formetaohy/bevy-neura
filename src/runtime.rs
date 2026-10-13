use bevy::prelude::Resource;
use neura::{HostIdentity, Runtime};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct NeuraRuntime {
    runtime: Arc<Runtime>,
    host: wgpu::Device,
    identity: HostIdentity,
}

impl NeuraRuntime {
    pub(crate) fn borrowed(host: wgpu::Device, runtime: Runtime, identity: HostIdentity) -> Self {
        Self {
            runtime: Arc::new(runtime),
            host,
            identity,
        }
    }

    pub fn host_device(&self) -> &wgpu::Device {
        &self.host
    }

    pub fn answers_to(&self, device: &wgpu::Device) -> bool {
        crate::host::identity(device) == self.identity
    }

    pub(crate) fn identity(&self) -> HostIdentity {
        self.identity
    }
}

impl Deref for NeuraRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Runtime {
        &self.runtime
    }
}
