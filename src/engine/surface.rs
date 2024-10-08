use ash::vk;
#[cfg(target_os = "macos")]
pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> Result<ash::vk::SurfaceKHR, crate::ApplicationError> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::os::raw::c_void;
    // use ash::mvk::macos_surface::Instance;

    let macos_instance = ash::mvk::macos_surface::Instance::new(entry, instance);

    use crate::ApplicationError;
    let handle = window.window_handle()
        .map_err(|e| ApplicationError::with_description("Failed to get window handle".to_string(), e.to_string()))?
        .as_raw();

    let RawWindowHandle::AppKit(handle) = handle else {
        return Err(ApplicationError::new("Failed to get macOS window handle".to_string()));
    };

    let ns_view = handle.ns_view.as_ptr() as *const c_void;

    let create_info = vk::MacOSSurfaceCreateInfoMVK {
        p_view: ns_view,
        ..Default::default()
    };
    
    let surface = unsafe {
        macos_instance.create_mac_os_surface(&create_info, None)?
    };
    Ok(surface)
}