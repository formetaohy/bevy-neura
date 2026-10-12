use bevy::prelude::Resource;
use neura::Runtime;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct NeuraRuntime {
    runtime: Arc<Runtime>,
    host: wgpu::Device,
}

impl NeuraRuntime {
    pub(crate) fn borrowed(host: wgpu::Device, runtime: Runtime) -> Self {
        Self {
            runtime: Arc::new(runtime),
            host,
        }
    }

    pub fn host_device(&self) -> &wgpu::Device {
        &self.host
    }
}

impl Deref for NeuraRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Runtime {
        &self.runtime
    }
}
