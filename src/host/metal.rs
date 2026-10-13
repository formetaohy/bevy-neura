use super::{lent, request};
use crate::runtime::NeuraRuntime;
use neura::{HostAdvance, HostDevice, HostIdentity, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::metal;

pub(super) fn identity(native: &metal::Device) -> HostIdentity {
    HostDevice::metal(native.raw_device()).identity()
}

pub(super) fn borrow(
    device: &Device,
    native: &metal::Device,
    memory: MemoryRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    let runtime = Runtime::borrow(
        lent(HostDevice::metal(native.raw_device()), device, advance),
        request(memory),
    )
    .unwrap_or_else(|unavailable| {
        panic!("the Metal device the App renders on cannot run Neura compute: {unavailable}")
    });
    NeuraRuntime::borrowed(device.clone(), runtime)
}
