use super::{borrowed, lent};
use crate::runtime::NeuraRuntime;
use neura::gpu::VulkanHost;
use neura::{HostAdvance, HostDevice, HostIdentity, RuntimeRequest};
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
    request: RuntimeRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    borrowed(
        lent(HostDevice::vulkan(host(device, native)), device, advance),
        request,
    )
}
