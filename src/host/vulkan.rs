use super::request;
use crate::runtime::NeuraRuntime;
use neura::gpu::VulkanHost;
use neura::{HostDevice, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::vulkan;

pub(super) fn borrow(
    device: &Device,
    native: &vulkan::Device,
    memory: MemoryRequest,
) -> NeuraRuntime {
    let instance = native.shared_instance();
    let host = VulkanHost {
        entry: instance.entry(),
        instance: instance.raw_instance(),
        physical: native.raw_physical_device(),
        device: native.raw_device(),
        queue: native.raw_queue(),
        queue_family: native.queue_family_index(),
        cooperative_matrix: device
            .features()
            .contains(wgpu::Features::EXPERIMENTAL_COOPERATIVE_MATRIX),
    };
    let runtime =
        Runtime::borrow(HostDevice::Vulkan(host), request(memory)).unwrap_or_else(|unavailable| {
            panic!("the Vulkan device the App renders on cannot run Neura compute: {unavailable}")
        });
    NeuraRuntime::borrowed(device.clone(), runtime)
}
