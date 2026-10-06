use crate::handle::NeuraHandle;
use crate::report::NeuraReport;
use crate::system;
use bevy::prelude::{App, Plugin, PreStartup, PreUpdate};
use neura::{GpuRequest, RuntimeRequest};

pub struct NeuraPlugin {
    gpu: GpuRequest,
    heap_bytes: u64,
    readback_bytes: u64,
}

impl NeuraPlugin {
    pub fn new(request: RuntimeRequest) -> Self {
        Self {
            gpu: request.gpu,
            heap_bytes: request.heap_bytes,
            readback_bytes: request.readback_bytes,
        }
    }
}

impl Default for NeuraPlugin {
    fn default() -> Self {
        Self::new(RuntimeRequest::default())
    }
}

impl Plugin for NeuraPlugin {
    fn build(&self, app: &mut App) {
        let request = RuntimeRequest {
            gpu: self.gpu.clone(),
            heap_bytes: self.heap_bytes,
            readback_bytes: self.readback_bytes,
        };
        app.insert_resource(NeuraHandle::spawn(request))
            .add_message::<NeuraReport>()
            .add_systems(PreStartup, system::open_device)
            .add_systems(PreUpdate, system::drain_reports);
    }
}
