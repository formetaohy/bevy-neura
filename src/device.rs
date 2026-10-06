use bevy::prelude::Resource;
use neura::{AdapterInfo, Runtime};

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct NeuraDevice {
    pub adapter: AdapterInfo,
    pub heap_bytes: u64,
    pub readback_bytes: u64,
    pub alignment: u64,
}

impl NeuraDevice {
    pub(crate) fn of(runtime: &Runtime) -> Self {
        Self {
            adapter: runtime.context().device().adapter_info().clone(),
            heap_bytes: runtime.heap_bytes(),
            readback_bytes: runtime.readback_capacity(),
            alignment: runtime.alignment(),
        }
    }
}
