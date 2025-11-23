use itertools::Itertools;
use std::{collections::HashSet, ffi::CStr};

use crate::{
    utils::{self},
    ApplicationError,
};
use ash::vk;

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
    pub fn new(
        entry: ash::Entry,
        app_info: vk::ApplicationInfo,
        instance_extensions: &[&CStr],
        layers_required: &[&CStr],
    ) -> Result<Self, ApplicationError> {
        if !Self::check_validation_layers(&entry, layers_required)? {
            let available_layers = unsafe { entry.enumerate_instance_layer_properties() }?;
            let available_layers = available_layers
                .into_iter()
                .map(|p| {
                    p.layer_name_as_c_str()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                })
                .intersperse(", ".to_string())
                .collect::<String>();
            return Err(ApplicationError::with_description(
                "A required layer isn't found".to_string(),
                format!("avvailable layers: {available_layers}"),
            ));
        }
        let instance_extensions = utils::slice_cstr_to_ptr(instance_extensions);
        let layers_required = utils::slice_cstr_to_ptr(layers_required);
        // TODO: add validation layers
        // entry.enumerate_instance_layer_properties()
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: instance_extensions.len() as u32,
            pp_enabled_extension_names: instance_extensions.as_ptr(),
            enabled_layer_count: layers_required.len() as u32,
            pp_enabled_layer_names: layers_required.as_ptr(),
            flags: Self::get_instance_flags(),
            ..Default::default()
        };
        let base = unsafe { entry.create_instance(&create_info, None)? };
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
    fn check_validation_layers(
        entry: &ash::Entry,
        layers_required: &[&CStr],
    ) -> Result<bool, ApplicationError> {
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }?;

        let mut available_layers = available_layers
            .iter()
            .map(|p| p.layer_name_as_c_str())
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|e| {
                ApplicationError::with_description(
                    "Unable to decode a layer name".to_string(),
                    e.to_string(),
                )
            })?;

        for req in layers_required {
            if !available_layers.remove(req) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Drop for Instances {
    fn drop(&mut self) {
        unsafe {
            self.base.destroy_instance(None);
        }
    }
}
