use ash::vk;

use crate::engine::builder;

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
            let capabilities = instances
                .surface
                .get_physical_device_surface_capabilities(*physical_device, *surface)?;
            let formats = instances
                .surface
                .get_physical_device_surface_formats(*physical_device, *surface)?;
            let present_modes = instances
                .surface
                .get_physical_device_surface_present_modes(*physical_device, *surface)?;
            Ok(SwapchainSupport {
                capabilities,
                formats,
                present_modes,
            })
        }
    }
    pub fn choose_swapchain_format(&self, expected: vk::Format) -> &vk::SurfaceFormatKHR {
        self.formats
            .iter()
            .find(|format| format.format == expected)
            .unwrap_or(self.formats.first().expect("no swapchain format available"))
    }
    pub fn choose_swapchain_present_mode(
        &self,
        expected: vk::PresentModeKHR,
    ) -> vk::PresentModeKHR {
        self.present_modes
            .contains(&expected)
            .then_some(expected)
            .unwrap_or(vk::PresentModeKHR::FIFO)
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

pub struct Swapchain {
    device: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_info: SwapchainInfo,
    pub images: Vec<vk::Image>,
    pub extent: vk::Extent2D,
}

#[derive(Clone, Copy, Debug)]
pub struct SwapchainInfo {
    pub format: vk::Format,
    pub present_mode: vk::PresentModeKHR,
}

impl Default for SwapchainInfo {
    fn default() -> Self {
        Self {
            format: vk::Format::B8G8R8A8_SRGB,
            present_mode: vk::PresentModeKHR::MAILBOX,
        }
    }
}

impl Swapchain {
    pub fn new(
        instances: &super::instances::Instances,
        device: &ash::Device,
        swapchain_support: SwapchainSupport,
        surface: &vk::SurfaceKHR,
        extent: vk::Extent2D,
        swapchain_info: SwapchainInfo,
    ) -> Result<Swapchain, vk::Result> {
        let swapchain_device = ash::khr::swapchain::Device::new(&instances.base, &device);
        let (swapchain, swapchain_info, extent) = Self::create_swapchain(
            &swapchain_device,
            swapchain_support,
            surface,
            extent,
            None,
            swapchain_info,
        )?;

        let images = unsafe { swapchain_device.get_swapchain_images(swapchain)? };
        Ok(Swapchain {
            device: swapchain_device,
            swapchain,
            swapchain_info,
            images,
            extent,
        })
    }

    fn create_swapchain(
        device: &ash::khr::swapchain::Device,
        swapchain_support: SwapchainSupport,
        surface: &vk::SurfaceKHR,
        extent: vk::Extent2D,
        old_swapchain: Option<vk::SwapchainKHR>,
        swapchain_info: SwapchainInfo,
    ) -> Result<(vk::SwapchainKHR, SwapchainInfo, vk::Extent2D), vk::Result> {
        let surface_format = swapchain_support.choose_swapchain_format(swapchain_info.format);
        let present_mode =
            swapchain_support.choose_swapchain_present_mode(swapchain_info.present_mode);
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
            Ok((
                device.create_swapchain(&create_infos, None)?,
                SwapchainInfo {
                    format: surface_format.format,
                    present_mode,
                },
                extent,
            ))
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_swapchain(self.swapchain, None);
        }
    }
}
