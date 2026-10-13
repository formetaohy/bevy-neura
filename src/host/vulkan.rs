use super::{lent, request};
use crate::runtime::NeuraRuntime;
use neura::gpu::VulkanHost;
use neura::{HostAdvance, HostDevice, HostIdentity, MemoryRequest, Runtime};
use wgpu::Device;
use wgpu::hal::vulkan;

fn host<'a>(device: &Device, native: &'a vulkan::Device) -> VulkanHost<'a> {
    let instance = native.shared_instance();
    VulkanHost {
        entry: instance.entry(),
        instance: instance.raw_instance(),
        physical: native.raw_physical_device(),
        device: native.raw_device(),
        queue: native.raw_queue(),
        queue_family: native.queue_family_index(),
        cooperative_matrix: device
            .features()
            .contains(wgpu::Features::EXPERIMENTAL_COOPERATIVE_MATRIX),
    }
}

pub(super) fn identity(device: &Device, native: &vulkan::Device) -> HostIdentity {
    HostDevice::vulkan(host(device, native)).identity()
}

pub(super) fn borrow(
    device: &Device,
    native: &vulkan::Device,
    memory: MemoryRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    let host = lent(HostDevice::vulkan(host(device, native)), device, advance);
    let identity = host.identity();
    let runtime = Runtime::borrow(host, request(memory)).unwrap_or_else(|unavailable| {
        panic!("the Vulkan device the App renders on cannot run Neura compute: {unavailable}")
    });
    NeuraRuntime::borrowed(device.clone(), runtime, identity)
}
