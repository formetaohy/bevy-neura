use crate::runtime::NeuraRuntime;
use neura::{HostAdvance, HostDevice, HostIdentity, Runtime, RuntimeRequest};
use std::any::Any;
use std::sync::Arc;
use wgpu::Device;

#[cfg(target_os = "windows")]
mod dx12;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod metal;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
mod vulkan;

pub(crate) fn identity(device: &Device) -> HostIdentity {
    #[cfg(target_os = "windows")]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Dx12>() } {
        return dx12::identity(&native);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Metal>() } {
        return metal::identity(&native);
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() } {
        return vulkan::identity(device, &native);
    }
    panic!(
        "Neura computes on the native device of a DX12, Metal or Vulkan renderer, and the render device of the App answers to none of them",
    )
}

pub(crate) fn borrow(
    device: &Device,
    request: RuntimeRequest,
    advance: Option<HostAdvance>,
) -> NeuraRuntime {
    #[cfg(target_os = "windows")]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Dx12>() } {
        return dx12::borrow(device, &native, request, advance);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Metal>() } {
        return metal::borrow(device, &native, request, advance);
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() } {
        return vulkan::borrow(device, &native, request, advance);
    }
    panic!(
        "Neura computes on the native device of a DX12, Metal or Vulkan renderer, and the render device of the App answers to none of them",
    )
}

pub(super) fn lent<'a>(
    host: HostDevice<'a>,
    device: &Device,
    advance: Option<HostAdvance>,
) -> HostDevice<'a> {
    let host = match advance {
        Some(advance) => host.advancing(advance),
        None => host,
    };
    let keep_alive: Arc<dyn Any + Send + Sync> = Arc::new(device.clone());
    host.holding(keep_alive)
}

pub(super) fn borrowed(host: HostDevice<'_>, request: RuntimeRequest) -> NeuraRuntime {
    let identity = host.identity();
    let runtime = Runtime::borrow(host, request).unwrap_or_else(|unavailable| {
        panic!("the render device of the App cannot run Neura compute: {unavailable}")
    });
    NeuraRuntime::borrowed(runtime, identity)
}
