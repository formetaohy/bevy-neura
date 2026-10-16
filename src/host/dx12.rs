use super::{borrowed, lent};
use crate::runtime::NeuraRuntime;
use neura::{HostAdvance, HostDevice, HostIdentity, RuntimeRequest};
use wgpu::Device;
use wgpu::hal::dx12;

pub(super) fn identity(native: &dx12::Device) -> HostIdentity {
    HostDevice::dx12(native.raw_device()).identity()
}

pub(super) fn borrow(
    device: &Device,
    native: &dx12::Device,
    request: RuntimeRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    borrowed(
        lent(HostDevice::dx12(native.raw_device()), device, advance),
        request,
    )
}
