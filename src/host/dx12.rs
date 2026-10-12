use super::request;
use crate::runtime::NeuraRuntime;
use neura::{HostDevice, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::dx12;

pub(super) fn borrow(
    device: &Device,
    native: &dx12::Device,
    memory: MemoryRequest,
) -> NeuraRuntime {
    let runtime = Runtime::borrow(HostDevice::Dx12(native.raw_device()), request(memory))
        .unwrap_or_else(|unavailable| {
            panic!("the DX12 device the App renders on cannot run Neura compute: {unavailable}")
        });
    NeuraRuntime::borrowed(device.clone(), runtime)
}
