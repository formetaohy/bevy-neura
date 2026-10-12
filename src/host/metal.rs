use super::request;
use crate::runtime::NeuraRuntime;
use neura::{HostDevice, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::metal;

pub(super) fn borrow(
    device: &Device,
    native: &metal::Device,
    memory: MemoryRequest,
) -> NeuraRuntime {
    let runtime = Runtime::borrow(HostDevice::Metal(native.raw_device()), request(memory))
        .unwrap_or_else(|unavailable| {
            panic!("the Metal device the App renders on cannot run Neura compute: {unavailable}")
        });
    NeuraRuntime::borrowed(device.clone(), runtime)
}
