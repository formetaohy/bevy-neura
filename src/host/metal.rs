use super::{borrowed, lent};
use crate::runtime::NeuraRuntime;
use neura::{HostAdvance, HostDevice, HostIdentity, RuntimeRequest};
use wgpu::Device;
use wgpu::hal::metal;

pub(super) fn identity(native: &metal::Device) -> HostIdentity {
    HostDevice::metal(native.raw_device()).identity()
}

pub(super) fn borrow(
    device: &Device,
    native: &metal::Device,
    request: RuntimeRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    borrowed(
        lent(HostDevice::metal(native.raw_device()), device, advance),
        request,
    )
}
