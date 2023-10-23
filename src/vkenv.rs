use std::sync::Arc;
use vulkano as vk;

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug)]
pub struct Queues {
    pub graphics: Arc<vk::device::Queue>,
    pub compute: Option<Arc<vk::device::Queue>>,
}

#[derive(Debug)]
pub struct VulkanEnvironment {
    pub instance: Arc<vk::instance::Instance>,
    pub physical_device: Arc<vk::device::physical::PhysicalDevice>,
    pub device: Arc<vk::device::Device>,
    pub queues: Queues,
    pub memory_allocator: vk::memory::allocator::GenericMemoryAllocator<Arc<vk::memory::allocator::FreeListAllocator>>,
    pub command_buffer_allocator: vk::command_buffer::allocator::StandardCommandBufferAllocator,
    pub window: Arc<winit::window::Window>,
    pub surface: Arc<vk::swapchain::Surface>,
}

impl VulkanEnvironment {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self> {
        let library = vk::VulkanLibrary::new()?;
        let required_extensions = vulkano_win::required_extensions(&library);
        let instance = vk::instance::Instance::new(
            library,
            vk::instance::InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )?;
        let (window, surface) = Self::new_window(event_loop, &instance)?;

        let device_extensions = vk::device::DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = Self::new_physical_device(
            &instance, 
            &surface,
            &device_extensions
        )?;
        let (logical_device, queues) = Self::new_logical_device(
            physical_device.clone(), 
            queue_family_index,
            device_extensions,
        )?;
        let queues = Queues { 
            graphics: queues.first().cloned().ok_or_else(|| anyhow::anyhow!("failed to find graphics queue"))?, 
            compute: None
        };
        let memory_allocator = vk::memory::allocator::StandardMemoryAllocator::new_default(logical_device.clone());
        let command_memory_allocator = vk::command_buffer::allocator::StandardCommandBufferAllocator::new(logical_device.clone(), Default::default());

        Ok(VulkanEnvironment {
            instance,
            physical_device,
            device: logical_device,
            queues,
            memory_allocator,
            command_buffer_allocator: command_memory_allocator,
            window,
            surface,
        })
    }
    fn new_window(event_loop: &winit::event_loop::EventLoop<()>, instance: &Arc<vk::instance::Instance>) -> Result<(Arc<winit::window::Window>, Arc<vk::swapchain::Surface>)> {
        use vulkano_win::VkSurfaceBuild;
        let surface = winit::window::WindowBuilder::new()
            .with_resizable(false)
            .build_vk_surface(event_loop, instance.clone())
            .map_err(|_| anyhow::anyhow!("failed to create window"))?;
        let window = surface
            .object()
            .ok_or_else(|| anyhow::anyhow!("failed to get window"))?
            .clone()
            .downcast::<winit::window::Window>()
            .map_err(|_| anyhow::anyhow!("failed to downcast window"))?;
        Ok((window, surface))
    }
    fn new_physical_device(
        instance: &Arc<vk::instance::Instance>,
        surface: &vk::swapchain::Surface,
        required_extensions: &vk::device::DeviceExtensions,
    ) -> Result<(Arc<vk::device::physical::PhysicalDevice>, u32)> {
        use vk::device::physical::PhysicalDeviceType;
        instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(required_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    // Find the first first queue family that is suitable.
                    // If none is found, `None` is returned to `filter_map`,
                    // which disqualifies this physical device.
                    .position(|(i, q)| {
                        q.queue_flags.contains(vk::device::QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .ok_or_else(|| anyhow::anyhow!("no suitable physical device found"))
    }
    fn new_logical_device(
        physical_device: Arc<vk::device::physical::PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: vk::device::DeviceExtensions,
    ) -> Result<(Arc<vk::device::Device>, Vec<Arc<vk::device::Queue>>)> {
        let (logical_device, queues) = vk::device::Device::new(
            physical_device,
            vk::device::DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![vk::device::QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;
        Ok((logical_device, queues.collect()))
    }
    fn new_swapchain(
        window: &Arc<winit::window::Window>,
        physical_device: &Arc<vk::device::physical::PhysicalDevice>,
        logical_device: &Arc<vk::device::Device>,
        surface: &Arc<vk::swapchain::Surface>,
    ) -> Result<(Arc<vk::swapchain::Swapchain>, Vec<Arc<vk::image::swapchain::SwapchainImage>>)> {
        let caps = physical_device
            .surface_capabilities(surface, Default::default())
            .map_err(|e| anyhow::anyhow!("failed to get surface capabilities: {}", e))?;
        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = Some(
            physical_device
                .surface_formats(surface, Default::default())
                .map_err(|e| anyhow::anyhow!("failed to get surface formats: {}", e))?[0]
                .0,
        );
        vk::swapchain::Swapchain::new(
            logical_device.clone(),
            surface.clone(),
            vk::swapchain::SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: vk::image::ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                composite_alpha,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("failed to create swapchain: {}", e))
    }
}

