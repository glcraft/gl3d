mod surface;
mod instances;
use std::collections::HashSet;

use ash::{vk, Entry};
use crate::error::ApplicationError;

pub struct Engine {
    pub instances: instances::Instances,
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queues: Queues,
    pub surface: vk::SurfaceKHR,
}

pub struct Queues {
    pub graphics: vk::Queue,
    pub present: vk::Queue,
    // pub compute: vk::Queue,
}

#[derive(Clone, Copy, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
    // compute: u32,
}

impl Engine {
    pub fn new(window: &winit::window::Window) -> Result<Engine, ApplicationError> {
        let entry = Self::init_entry()?;

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 4, 0),
            application_version: 1,
            engine_version: 1,
            p_engine_name: "Vulkan Engine".as_ptr() as *const i8,
            p_application_name: "Vulkan App".as_ptr() as *const i8,
            ..Default::default()
        };

        let instances = instances::Instances::new(entry, app_info)?;

        let surface = surface::create_surface(&instances, window)?;

        let physical_device = Self::get_physical_device(&instances)?;
        
        let queue_indices = Self::get_queue_family_indices(&instances, &physical_device, &surface)?;
        
        let logical_device = Self::create_logical_device(&instances.base, physical_device, queue_indices)?;
        let queues = Queues {
            graphics: unsafe { logical_device.get_device_queue(queue_indices.graphics, 0) },
            present: unsafe { logical_device.get_device_queue(queue_indices.present, 0) },
            // compute: unsafe { logical_device.get_device_queue(queue_indices.compute, 0) },
        };
        Ok(Engine {
            instances,
            physical_device,
            logical_device,
            queues,
            surface
        })
    }

    fn init_entry() -> Result<Entry, ApplicationError> {
        unsafe { Ok(Entry::load_from("/opt/homebrew/lib/libvulkan.1.dylib")?) }
    }

    fn get_physical_device(instances: &instances::Instances) -> Result<vk::PhysicalDevice, ApplicationError> {
        let physical_devices = unsafe { instances.base.enumerate_physical_devices()? };
        let physical_device = physical_devices
            .into_iter()
            .find(|d| Self::check_device(instances, d))
            .ok_or("No suitable device found")?;
        Ok(physical_device)
    }

    fn create_logical_device(instance: &ash::Instance, device: vk::PhysicalDevice, queue_indices: QueueFamilyIndices) -> Result<ash::Device, ApplicationError> {
        let features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };
        let extensions = &[];

        let mut queue_set = HashSet::with_capacity(3);
        queue_set.insert(queue_indices.graphics);
        queue_set.insert(queue_indices.present);
        // queue_set.insert(queue_indices.compute);
        let queue_create_infos = queue_set
            .into_iter()
            .map(|i| vk::DeviceQueueCreateInfo {
                queue_family_index: i,
                queue_count: 1,
                p_queue_priorities: &1.0,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let device_create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr() as *const _,
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            p_enabled_features: &features,
            ..Default::default()
        };

        let device = unsafe { instance.create_device(device, &device_create_info, None)? };
        Ok(device)
    }
    fn check_device(instances: &instances::Instances, device: &ash::vk::PhysicalDevice) -> bool {
        unsafe {
            let properties = instances.base.get_physical_device_properties(*device);
            let _features = instances.base.get_physical_device_features(*device);
            // let surface_ = instances.surface.get_physical_device_surface_c(*device, 0, *instances.surface).unwrap();
            properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU || properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
        }
    }
    fn get_queue_family_indices(instances: &instances::Instances, device: &ash::vk::PhysicalDevice, surface: &ash::vk::SurfaceKHR) -> Result<QueueFamilyIndices, ApplicationError> {
        let queue_family_properties = unsafe { instances.base.get_physical_device_queue_family_properties(*device) };
        let mut graphics = None;
        let mut present = None;
        // let mut compute = None;
        for (i, queue_family) in queue_family_properties.iter().enumerate().map(|(i, p)| (i as u32, p)) {
            if graphics.is_none() && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(i);
            }
            if present.is_none() && unsafe { instances.surface.get_physical_device_surface_support(*device, i, *surface)? } {
                present = Some(i);
            }
            // if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            //     compute = Some(i as u32);
            // }
        }
        ;
        Ok(QueueFamilyIndices {
            graphics: graphics.ok_or("No graphics queue family found")?,
            present: present.ok_or("No present queue family found")?,
            // compute: compute.ok_or("No compute queue family found")?,
        })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_device(None);
            self.instances.surface.destroy_surface(self.surface, None);
        }
    }
}