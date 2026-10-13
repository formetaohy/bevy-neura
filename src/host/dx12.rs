use super::{lent, request};
use crate::runtime::NeuraRuntime;
use neura::{HostAdvance, HostDevice, HostIdentity, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::dx12;

pub(super) fn identity(native: &dx12::Device) -> HostIdentity {
    HostDevice::dx12(native.raw_device()).identity()
}

pub(super) fn borrow(
    device: &Device,
    native: &dx12::Device,
    memory: MemoryRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    let host = lent(HostDevice::dx12(native.raw_device()), device, advance);
    let identity = host.identity();
    let runtime = Runtime::borrow(host, request(memory)).unwrap_or_else(|unavailable| {
        panic!("the DX12 device the App renders on cannot run Neura compute: {unavailable}")
    });
    NeuraRuntime::borrowed(device.clone(), runtime, identity)
}
