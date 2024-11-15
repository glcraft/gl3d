use std::fmt::write;

use ash::vk;

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn new(
        instances: &super::instances::Instances,
        physical_device: &vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> Result<SwapchainSupport, vk::Result> {
        unsafe {
            let capabilities = instances.surface.get_physical_device_surface_capabilities(*physical_device, *surface)?;
            let formats = instances.surface.get_physical_device_surface_formats(*physical_device, *surface)?;
            let present_modes = instances.surface.get_physical_device_surface_present_modes(*physical_device, *surface)?;
            Ok(SwapchainSupport {
                capabilities,
                formats,
                present_modes,
            })
        }
    }
    pub fn choose_swapchain_format(&self) -> &vk::SurfaceFormatKHR {
        self.formats.iter()
            .find(|format| format.format == vk::Format::B8G8R8A8_SRGB)
            .unwrap_or(self.formats.first().expect("no swapchain format available"))
    }
    pub fn choose_swapchain_present_mode(&self) -> vk::PresentModeKHR {
        if self.present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }
    pub fn clamp_extent(&self, extent: vk::Extent2D) -> vk::Extent2D {
        vk::Extent2D {
            width: extent.width.clamp(
                self.capabilities.min_image_extent.width,
                self.capabilities.max_image_extent.width,
            ),
            height: extent.height.clamp(
                self.capabilities.min_image_extent.height,
                self.capabilities.max_image_extent.height,
            ),
        }
    }
}

pub fn create_swapchain(
    instances: &super::instances::Instances,
    device: &ash::Device,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
    extent: vk::Extent2D,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> Result<vk::SwapchainKHR, vk::Result> {
    let swapchain_support = SwapchainSupport::new(instances, physical_device, surface)?;
    let surface_format = swapchain_support.choose_swapchain_format();
    let present_mode = swapchain_support.choose_swapchain_present_mode();
    let extent = swapchain_support.clamp_extent(extent);
    let image_count = (swapchain_support.capabilities.min_image_count + 1)
        .min(swapchain_support.capabilities.max_image_count as u32)
        .max(if swapchain_support.capabilities.max_image_count > 0 {
            swapchain_support.capabilities.max_image_count
        } else {
           u32::max_value() 
        });
    let create_infos = vk::SwapchainCreateInfoKHR {
        surface: *surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: std::ptr::null(),
        pre_transform: swapchain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: old_swapchain.unwrap_or(vk::SwapchainKHR::null()),
        ..Default::default()
    };
    unsafe {
        let device = ash::khr::swapchain::Device::new(&instances.base, &device);
        device.create_swapchain(&create_infos, None)
    }
}
