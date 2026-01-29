use ash::khr::swapchain::Device as SwapchainDevice;
use ash::prelude::*;
use ash::vk::{self, SwapchainKHR};

pub struct Builder<'a> {
    // Requirted
    device: &'a ash::Device,
    instance: &'a ash::Instance,
    surface_instance: &'a ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    // Optional
    image_format: Option<vk::SurfaceFormatKHR>,
    image_color_space: Option<vk::ColorSpaceKHR>,
    present_mode: vk::PresentModeKHR,
    depth_stencil: Option<vk::SurfaceFormatKHR>,
    old_swapchain: Option<vk::SwapchainKHR>,
    // Geenrated
    capabilities: Option<vk::SurfaceCapabilitiesKHR>,
}

impl<'a> Builder<'a> {
    pub fn new(
        device: &'a ash::Device,
        instances: &'a super::instances::Instances,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Self {
        Self {
            device,
            instance: &instances.base,
            surface_instance: &instances.surface,
            physical_device,
            surface,
            image_format: None,
            image_color_space: None,
            present_mode: vk::PresentModeKHR::FIFO,
            depth_stencil: None,
            old_swapchain: None,
            capabilities: None,
        }
    }

    pub fn build(mut self) -> VkResult<Swapchain> {
        let swapchain_device = SwapchainDevice::new(self.instance, self.device);

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: self.surface,
            min_image_count: self.get_capabilities()?.min_image_count,
            image_format: todo!(),
            image_color_space: todo!(),
            image_extent: todo!(),
            image_array_layers: todo!(),
            image_usage: todo!(),
            image_sharing_mode: todo!(),
            queue_family_index_count: todo!(),
            p_queue_family_indices: todo!(),
            pre_transform: todo!(),
            composite_alpha: todo!(),
            present_mode: self.present_mode,
            clipped: vk::TRUE,
            old_swapchain: self.old_swapchain.unwrap_or_else(vk::SwapchainKHR::null),
            ..Default::default()
        };

        let swapchain = unsafe { swapchain_device.create_swapchain(&create_info, None)? };

        Ok(Swapchain {
            swapchain_device,
            swapchain,
            swapchain_info: todo!(),
            images: todo!(),
            extent: todo!(),
        })
    }
    fn get_capabilities(&mut self) -> VkResult<&vk::SurfaceCapabilitiesKHR> {
        if self.capabilities.is_none() {
            self.capabilities = Some(unsafe {
                self.surface_instance
                    .get_physical_device_surface_capabilities(self.physical_device, self.surface)?
            })
        }
        Ok(unsafe { self.capabilities.as_ref().unwrap_unchecked() })
    }
}

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
    ) -> VkResult<SwapchainSupport> {
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
    swapchain_device: SwapchainDevice,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_info: SwapchainInfo,
    pub images: Vec<(vk::Image, vk::ImageView)>,
    pub extent: vk::Extent2D,
}

#[derive(Clone, Copy, Debug)]
pub struct SwapchainInfo {
    pub format: vk::Format,
    pub present_mode: vk::PresentModeKHR,
    pub enable_depth_stencil: bool,
}

impl Default for SwapchainInfo {
    fn default() -> Self {
        Self {
            format: vk::Format::B8G8R8A8_SRGB,
            present_mode: vk::PresentModeKHR::MAILBOX,
            enable_depth_stencil: false,
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
    ) -> VkResult<Swapchain> {
        let swapchain_device = SwapchainDevice::new(&instances.base, &device);
        let (swapchain, swapchain_info, extent) = Self::create_swapchain(
            &swapchain_device,
            swapchain_support,
            surface,
            extent,
            None,
            swapchain_info,
        )?;

        let mut images =
            Self::create_images(device, &swapchain_device, &swapchain, swapchain_info.format)?;
        if swapchain_info.enable_depth_stencil {
            // images.push(Self::create_depth_stencil_attachment(device));
        }
        Ok(Swapchain {
            swapchain_device,
            swapchain,
            swapchain_info,
            images,
            extent,
        })
    }

    fn create_swapchain(
        device: &SwapchainDevice,
        swapchain_support: SwapchainSupport,
        surface: &vk::SurfaceKHR,
        extent: vk::Extent2D,
        old_swapchain: Option<vk::SwapchainKHR>,
        swapchain_info: SwapchainInfo,
    ) -> VkResult<(vk::SwapchainKHR, SwapchainInfo, vk::Extent2D)> {
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
        // TODO: Configurable image sharing
        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain#page_Creating-the-swap-chain
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
                    enable_depth_stencil: swapchain_info.enable_depth_stencil,
                },
                extent,
            ))
        }
    }
    fn create_images(
        device: &ash::Device,
        swapchain_device: &SwapchainDevice,
        swapchain: &SwapchainKHR,
        swapchain_image_format: vk::Format,
    ) -> VkResult<Vec<(vk::Image, vk::ImageView)>> {
        let images = unsafe { swapchain_device.get_swapchain_images(*swapchain) }?;
        images
            .into_iter()
            .map(|image| {
                let create_info = vk::ImageViewCreateInfo {
                    image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format: swapchain_image_format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                Ok((image, unsafe {
                    device.create_image_view(&create_info, None)
                }?))
            })
            .collect()
    }
    fn create_depth_stencil_attachment(
        device: &ash::Device,
    ) -> VkResult<(vk::Image, vk::ImageView)> {
        // for format in [vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT] {
        //     device.
        // }
        todo!()
    }
    pub fn drop_with(&mut self, device: &ash::Device) {
        unsafe {
            while let Some(image) = self.images.pop() {
                device.destroy_image_view(image.1, None);
            }
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        if !self.images.is_empty() {
            panic!("Swapchain not dropped by `Swapchain::drop_with`")
        }
    }
}
