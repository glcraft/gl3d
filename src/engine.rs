pub mod builder;
mod instances;
mod pipeline;
mod render_pass;
mod shader;
mod surface;
mod swapchain;

use crate::{error::ApplicationError, utils::IteratorTryAny};
use ash::{vk, Entry};
pub use builder::Builder as EngineBuilder;
use std::{
    collections::HashSet,
    ffi::{c_void, CStr},
};

pub struct Engine {
    pub instances: instances::Instances,
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queues: Queues,
    pub allocator: vk_mem::Allocator,
    pub surface: Option<vk::SurfaceKHR>,
    pub swapchain: Option<swapchain::Swapchain>,
}

pub struct Queues {
    pub graphics: Option<vk::Queue>,
    // pub present: vk::Queue,
    pub compute: Option<vk::Queue>,
}

#[derive(Default, Clone, Copy, Debug)]
struct QueueFamilyIndices {
    graphics: Option<u32>,
    // present: Option<u32>,
    compute: Option<u32>,
}

impl Engine {
    pub fn new(builder: EngineBuilder) -> Result<Engine, ApplicationError> {
        let instances = Self::make_instances(
            builder.app_info.clone().unwrap_or_default().into(),
            &builder.instance_extensions,
            &builder.layers,
        )?;
        Self::init_engine(instances, builder, None)
    }
    pub fn with_window(
        mut builder: EngineBuilder,
        window: &winit::window::Window,
    ) -> Result<Engine, ApplicationError> {
        let instances = Self::make_instances(
            builder.app_info.clone().unwrap_or_default().into(),
            &builder.instance_extensions,
            &builder.layers,
        )?;
        builder.queue_families.present = true;
        let surface = Some(surface::create_surface(&instances, window)?);
        Self::init_engine(instances, builder, surface)
    }

    fn init_engine(
        instances: instances::Instances,
        builder: EngineBuilder,
        surface: Option<vk::SurfaceKHR>,
    ) -> Result<Engine, ApplicationError> {
        let physical_device = Self::get_physical_device(&instances, &builder.device_extensions)?;

        let queue_indices = Self::get_queue_family_indices(
            &instances,
            &physical_device,
            &builder.queue_families,
            surface.as_ref(),
        )?;
        let logical_device = Self::create_logical_device(
            &instances.base,
            physical_device,
            queue_indices,
            &builder.queue_families,
            &builder.device_extensions,
        )?;

        let allocator = Self::make_allocator(physical_device, &logical_device, &instances.base)?;

        let queues = Queues {
            graphics: queue_indices
                .graphics
                .map(|graphics| unsafe { logical_device.get_device_queue(graphics, 0) }),
            // present: unsafe { logical_device.get_device_queue(queue_indices.present, 0) },
            compute: queue_indices
                .compute
                .map(|compute| unsafe { logical_device.get_device_queue(compute, 0) }),
        };
        let swapchain = builder
            .extent
            .map(|extent| {
                let swapchain_support = swapchain::SwapchainSupport::new(
                    &instances,
                    &physical_device,
                    surface.as_ref().unwrap(),
                )?;
                swapchain::Swapchain::new(
                    &instances,
                    &logical_device,
                    swapchain_support,
                    surface.as_ref().unwrap(),
                    extent,
                    builder.swapchain_info,
                )
            })
            .transpose()?;
        Ok(Engine {
            instances,
            physical_device,
            logical_device,
            queues,
            surface,
            allocator,
            swapchain,
        })
    }

    fn make_instances(
        app_info: vk::ApplicationInfo,
        instance_extensions: &[&CStr],
        layers: &[&CStr],
    ) -> Result<instances::Instances, ApplicationError> {
        // let entry = Self::init_entry()?;
        let entry = unsafe { Entry::load() }?;
        instances::Instances::new(entry, app_info, instance_extensions, layers)
    }

    fn get_physical_device(
        instances: &instances::Instances,
        extensions: &[&CStr],
    ) -> Result<vk::PhysicalDevice, ApplicationError> {
        let physical_devices = unsafe { instances.base.enumerate_physical_devices()? };
        for physical_device in physical_devices {
            if Self::check_device(instances, &physical_device, extensions)? {
                return Ok(physical_device);
            }
        }
        Err(ApplicationError::new(
            "No suitable device found".to_string(),
        ))
    }
    fn query_swapchain_support(
        instances: &instances::Instances,
        physical_device: &vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> Result<swapchain::SwapchainSupport, ApplicationError> {
        swapchain::SwapchainSupport::new(instances, physical_device, surface)
            .map_err(|_| ApplicationError::new("Failed to query swapchain support".to_string()))
    }
    fn create_logical_device(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        queue_indices: QueueFamilyIndices,
        queue_families: &builder::QueueFamilies,
        extensions: &[&CStr],
    ) -> Result<ash::Device, ApplicationError> {
        let vk12_features = vk::PhysicalDeviceVulkan12Features {
            descriptor_indexing: vk::TRUE,
            descriptor_binding_variable_descriptor_count: vk::TRUE,
            runtime_descriptor_array: vk::TRUE,
            buffer_device_address: vk::TRUE,
            ..Default::default()
        };
        let vk13_features = vk::PhysicalDeviceVulkan13Features {
            p_next: std::ptr::from_ref(&vk12_features) as *mut c_void,
            synchronization2: vk::TRUE,
            dynamic_rendering: vk::TRUE,
            ..Default::default()
        };
        let vk10_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        };

        let mut queue_set = HashSet::with_capacity(3);
        if let Some(graphics) = queue_indices.graphics {
            queue_set.insert(graphics);
        }
        if let Some(compute) = queue_indices.compute {
            queue_set.insert(compute);
        }
        let queue_create_infos = queue_set
            .into_iter()
            .map(|i| vk::DeviceQueueCreateInfo {
                queue_family_index: i,
                queue_count: 1,
                p_queue_priorities: &1.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let extensions_ptr: Vec<*const i8> =
            extensions.iter().map(|&slice| slice.as_ptr()).collect();
        let device_create_info = vk::DeviceCreateInfo {
            p_next: std::ptr::from_ref(&vk13_features) as *mut c_void,
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr() as *const _,
            pp_enabled_extension_names: extensions_ptr.as_ptr() as *const _,
            enabled_extension_count: extensions.len() as u32,
            p_enabled_features: &vk10_features,
            ..Default::default()
        };

        let device = unsafe { instance.create_device(device, &device_create_info, None)? };
        Ok(device)
    }
    fn make_allocator(
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        instance: &ash::Instance,
    ) -> Result<vk_mem::Allocator, vk::Result> {
        let mut allocator_create_info =
            vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);
        allocator_create_info.flags = vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
        unsafe { vk_mem::Allocator::new(allocator_create_info) }
    }
    fn check_device(
        instances: &instances::Instances,
        device: &ash::vk::PhysicalDevice,
        extensions: &[&CStr],
    ) -> Result<bool, ApplicationError> {
        unsafe {
            let properties = instances.base.get_physical_device_properties(*device);
            let _features = instances.base.get_physical_device_features(*device);
            // let surface_ = instances.surface.get_physical_device_surface_c(*device, 0, *instances.surface).unwrap();

            Ok(Self::check_device_extension(instances, device, extensions)?
                && (properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                    || properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU))
        }
    }
    fn check_device_extension(
        instances: &instances::Instances,
        device: &vk::PhysicalDevice,
        extensions: &[&CStr],
    ) -> Result<bool, ApplicationError> {
        let properties = unsafe {
            instances
                .base
                .enumerate_device_extension_properties(*device)?
        };
        for required_extension in extensions {
            if !properties
                .iter()
                .map(|p| p.extension_name_as_c_str())
                .try_any(|ext| ext == required_extension)
                .map_err(|e| {
                    ApplicationError::with_description(
                        "Unable to décode device extension".to_string(),
                        e.to_string(),
                    )
                })?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn get_queue_family_indices(
        instances: &instances::Instances,
        device: &vk::PhysicalDevice,
        expected_queues: &builder::QueueFamilies,
        surface: Option<&vk::SurfaceKHR>,
    ) -> Result<QueueFamilyIndices, ApplicationError> {
        assert!(
            expected_queues.present && surface.is_some(),
            "Missing surface while checking for present queue family"
        );
        let queue_family_properties = unsafe {
            instances
                .base
                .get_physical_device_queue_family_properties(*device)
        };

        let mut graphics = None;
        let mut compute = None;
        for (i, queue_family) in queue_family_properties
            .iter()
            .enumerate()
            .map(|(i, p)| (i as u32, p))
        {
            if expected_queues.graphics.is_some()
                && graphics.is_none()
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && expected_queues.present
                && unsafe {
                    instances.surface.get_physical_device_surface_support(
                        *device,
                        i,
                        *surface.unwrap(),
                    )?
                }
            {
                graphics = Some(i);
            }
            if expected_queues.compute.is_some()
                && compute.is_none()
                && queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
            {
                compute = Some(i);
            }
            if graphics.is_some() && compute.is_some() {
                break;
            }
        }
        Ok(QueueFamilyIndices { graphics, compute })
    }
    #[inline]
    fn get_device_extensions() -> Vec<&'static [u8]> {
        vec![ash::khr::swapchain::NAME.to_bytes()]
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut a) = self.swapchain.take() {
                a.drop_with(&self.logical_device);
            }
            self.logical_device.destroy_device(None);
            if let Some(surface) = self.surface {
                self.instances.surface.destroy_surface(surface, None);
            }
            // if let Some(swapchain) = self.swapchain {
            //     let device = ash::khr::swapchain::Device::new(&instances.base, &device);
            //     device.destroy_swapchain(swapchain, None);
            // }
        }
    }
}
