use crate::runtime::NeuraRuntime;
use neura::{GpuRequest, MemoryRequest, RuntimeRequest};
use wgpu::Device;

#[cfg(target_os = "windows")]
mod dx12;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod metal;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
mod vulkan;

pub(crate) fn borrow(device: &Device, memory: MemoryRequest) -> NeuraRuntime {
    #[cfg(target_os = "windows")]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Dx12>() } {
        return dx12::borrow(device, &native, memory);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Metal>() } {
        return metal::borrow(device, &native, memory);
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
    if let Some(native) = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() } {
        return vulkan::borrow(device, &native, memory);
    }
    panic!(
        "Neura computes on the native device of a DX12, Metal or Vulkan renderer, and the render device of the App answers to none of them",
    )
}

pub(crate) fn request(memory: MemoryRequest) -> RuntimeRequest {
    RuntimeRequest {
        gpu: GpuRequest::default(),
        memory,
    }
}
