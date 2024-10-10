use ash::vk;
use crate::ApplicationError;

pub struct Instances {
    _entry: ash::Entry,
    pub base: ash::Instance,
    pub surface: ash::khr::surface::Instance,
    #[cfg(target_os = "macos")]
    pub metal_surface: ash::ext::metal_surface::Instance,
    #[cfg(target_os = "ios")]
    pub ios_surface: ash::mvk::ios_surface::Instance,
    #[cfg(target_os = "windows")]
    pub win32_surface: ash::khr::win32_surface::Instance,
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    pub wayland_surface: ash::khr::wayland_surface::Instance,
    #[cfg(all(target_os = "linux", not(feature = "wayland")))]
    pub xlib_surface: ash::khr::xlib_surface::Instance,
}

impl Instances {
    pub fn new(entry: ash::Entry, app_info: vk::ApplicationInfo) -> Result<Self, ApplicationError> {
        let instance_extensions = Self::get_instance_extensions();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: instance_extensions.len() as u32,
            pp_enabled_extension_names: instance_extensions.as_ptr(),
            flags: Self::get_instance_flags(),
            ..Default::default()
        };

        let base = unsafe {
            entry.create_instance(&create_info, None)?
        };
        let surface = ash::khr::surface::Instance::new(&entry, &base);
        #[cfg(target_os = "macos")]
        let macos_surface = ash::ext::metal_surface::Instance::new(&entry, &base);
        #[cfg(target_os = "ios")]
        let ios_surface = ash::mvk::ios_surface::Instance::new(&entry, &base);
        #[cfg(target_os = "windows")]
        let win32_surface = ash::khr::win32_surface::Instance::new(&entry, &base);
        #[cfg(all(target_os = "linux", feature = "wayland"))]
        let wayland_surface = ash::khr::wayland_surface::Instance::new(&entry, &base);
        #[cfg(all(target_os = "linux", not(feature = "wayland")))]
        let xlib_surface = ash::khr::xlib_surface::Instance::new(&entry, &base);
        Ok(Self {
            _entry: entry,
            base,
            surface,
            #[cfg(target_os = "macos")]
            metal_surface: macos_surface,
            #[cfg(target_os = "ios")]
            ios_surface,
            #[cfg(target_os = "windows")]
            win32_surface,
            #[cfg(all(target_os = "linux", feature = "wayland"))]
            wayland_surface,
            #[cfg(all(target_os = "linux", not(feature = "wayland")))]
            xlib_surface,
        })
    }
    #[inline]
    fn get_instance_flags() -> vk::InstanceCreateFlags {
        let mut flags = vk::InstanceCreateFlags::default();
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }
        flags
    }
    fn get_instance_extensions() -> Vec<*const i8> {
        let mut instance_extentions = Vec::with_capacity(10);
        instance_extentions.push(ash::khr::surface::NAME.as_ptr());
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        instance_extentions.push(ash::khr::portability_enumeration::NAME.as_ptr());
        #[cfg(target_os = "macos")]
        instance_extentions.push(ash::ext::metal_surface::NAME.as_ptr());
        #[cfg(target_os = "ios")]
        instance_extentions.push(ash::mvk::ios_surface::NAME.as_ptr());
        #[cfg(target_os = "windows")]
        instance_extentions.push(ash::khr::win32_surface::NAME.as_ptr());
        #[cfg(all(target_os = "linux", feature = "wayland"))]
        instance_extentions.push(ash::khr::wayland_surface::NAME.as_ptr());
        #[cfg(all(target_os = "linux", not(feature = "wayland")))]
        instance_extentions.push(ash::khr::xlib_surface::NAME.as_ptr());
        
        instance_extentions
    }
}

impl Drop for Instances {
    fn drop(&mut self) {
        unsafe {
            self.base.destroy_instance(None);
        }
    }
}