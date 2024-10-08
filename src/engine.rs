mod surface;
use ash::{vk, Entry};
use crate::error::ApplicationError;

pub struct Engine {
    pub entry: Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queues: Queues,
    pub window: winit::window::Window,
}

pub struct Queues {
    pub graphics: vk::Queue,
    // pub compute: vk::Queue,
}

#[derive(Clone, Copy, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
    // compute: u32,
}

impl Engine {
    pub fn new(window: winit::window::Window) -> Result<Engine, ApplicationError> {
        let entry = Self::init_entry()?;

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 4, 0),
            application_version: 1,
            engine_version: 1,
            p_engine_name: "Vulkan Engine".as_ptr() as *const i8,
            p_application_name: "Vulkan App".as_ptr() as *const i8,
            ..Default::default()
        };

        let instance = Self::init_instance(&entry, app_info)?;

        let physical_device = Self::get_physical_device(&instance)?;
        
        let queue_indices = Self::get_queue_family_indices(&instance, &physical_device)?;
        
        let logical_device = Self::create_logical_device(&instance, physical_device, queue_indices.graphics)?;
        let queues = Queues {
            graphics: unsafe { logical_device.get_device_queue(queue_indices.graphics, 0) },
            // compute: unsafe { logical_device.get_device_queue(queue_indices.compute, 0) },
        };
        Ok(Engine {
            entry,
            instance,
            physical_device,
            logical_device,
            queues,
            window,
        })
    }

    fn init_entry() -> Result<Entry, ApplicationError> {
        unsafe { Ok(Entry::load_from("/opt/homebrew/lib/libvulkan.1.dylib")?) }
    }
    fn init_instance(entry: &Entry, app_info: vk::ApplicationInfo) -> Result<ash::Instance, ApplicationError> {
        let instance_extensions = Self::get_instance_extensions();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: instance_extensions.len() as u32,
            pp_enabled_extension_names: instance_extensions.as_ptr(),
            flags: vk::InstanceCreateFlags::default() | vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(instance)
    }

    fn get_physical_device(instance: &ash::Instance) -> Result<vk::PhysicalDevice, ApplicationError> {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };
        let physical_device = physical_devices
            .into_iter()
            .find(|d| Self::check_device(instance, d))
            .ok_or("No suitable device found")?;
        Ok(physical_device)
    }

    fn create_logical_device(instance: &ash::Instance, device: vk::PhysicalDevice, queue_indices: u32) -> Result<ash::Device, ApplicationError> {
        let features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };
        let extensions = &[];

        let queue_create_infos = vk::DeviceQueueCreateInfo {
            queue_family_index: queue_indices,
            queue_count: 1,
            p_queue_priorities: &1.0,
            ..Default::default()
        };

        let device_create_info = vk::DeviceCreateInfo {
            p_queue_create_infos: &queue_create_infos as *const _,
            pp_enabled_extension_names: extensions.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            p_enabled_features: &features,
            ..Default::default()
        };

        let device = unsafe { instance.create_device(device, &device_create_info, None)? };
        Ok(device)
    }

    fn get_instance_extensions() -> Vec<*const i8> {
        let mut instance_extentions = Vec::with_capacity(10);
        instance_extentions.push(ash::khr::surface::NAME.as_ptr());
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        instance_extentions.push(ash::khr::portability_enumeration::NAME.as_ptr());
        #[cfg(target_os = "macos")]
        instance_extentions.push(ash::mvk::macos_surface::NAME.as_ptr());
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
    fn check_device(instance: &ash::Instance, device: &ash::vk::PhysicalDevice) -> bool {
        unsafe {
            let properties = instance.get_physical_device_properties(*device);
            let _features = instance.get_physical_device_features(*device);
            properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU || properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
        }
    }
    fn get_queue_family_indices(instance: &ash::Instance, device: &ash::vk::PhysicalDevice) -> Result<QueueFamilyIndices, ApplicationError> {
        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*device) };
        let mut graphics = None;
        // let mut compute = None;
        for (i, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(i as u32);
            }
            // if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            //     compute = Some(i as u32);
            // }
        }
        Ok(QueueFamilyIndices {
            graphics: graphics.ok_or("No graphics queue family found")?,
            // compute: compute.ok_or("No compute queue family found")?,
        })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}