use ash::vk;
use winit::raw_window_handle::{HasWindowHandle, HasDisplayHandle, RawWindowHandle, RawDisplayHandle};
use super::instances;
use crate::ApplicationError;

#[cfg(target_os = "macos")]
pub fn create_surface(instances: &instances::Instances, window: &winit::window::Window) -> Result<ash::vk::SurfaceKHR, ApplicationError> {
    // https://docs.rs/ash-window/latest/src/ash_window/lib.rs.html#98
    use std::os::raw::c_void;
    use raw_window_metal::Layer;
    
    let handle = window.window_handle()
        .map_err(|e| ApplicationError::with_description("Failed to get window handle".to_string(), e.to_string()))?;
    let handle = handle.as_raw();

    let RawWindowHandle::AppKit(handle) = handle else {
        return Err(ApplicationError::new("Failed to get macOS window handle".to_string()));
    };
    unsafe {
        let layer = Layer::from_ns_view(handle.ns_view);
        let create_info = vk::MetalSurfaceCreateInfoEXT {
            p_layer: layer.as_ptr().as_ptr() as *const c_void,
            ..Default::default()
        };
        
        let surface = instances.metal_surface.create_metal_surface(&create_info, None)?;
        Ok(surface)
    }
}

#[cfg(target_os = "linux")]
pub fn create_surface(instances: &instances::Instances, window: &winit::window::Window) -> Result<ash::vk::SurfaceKHR, ApplicationError> {
    use ash::vk::WaylandSurfaceCreateInfoKHR;
    let wl_surface = {
        let handle = window.window_handle()
            .map_err(|e| ApplicationError::with_description("Failed to get window handle".to_string(), e.to_string()))?
            .as_raw();
        let RawWindowHandle::Wayland(handle) = handle else {
            return Err(ApplicationError::new("Failed to get macOS window handle".to_string()));
        };
        handle.surface
    };
    let wl_display = {
        let handle = window.display_handle()
            .map_err(|e| ApplicationError::with_description("Failed to get window handle".to_string(), e.to_string()))?
            .as_raw();
        let RawDisplayHandle::Wayland(handle) = handle else {
            return Err(ApplicationError::new("Failed to get macOS window handle".to_string()));
        };
        handle.display
    };
    let create_info = WaylandSurfaceCreateInfoKHR::default().display(wl_display.as_ptr()).surface(wl_surface.as_ptr());
    Ok(unsafe { instances.wayland_surface.create_wayland_surface(&create_info, None)? })
}
