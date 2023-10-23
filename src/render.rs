use vulkano as vk;
use std::sync::Arc;
use crate::vkenv::VulkanEnvironment;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub struct Framebuffer {
    pub image: Arc<vk::image::swapchain::SwapchainImage>,
    pub framebuffer: Arc<vk::render_pass::Framebuffer>,
    pub command_buffer: Arc<vk::command_buffer::PrimaryAutoCommandBuffer>,
}

impl Framebuffer {
    pub fn new(render_pass: Arc<vk::render_pass::RenderPass>, image: Arc<vk::image::swapchain::SwapchainImage>) -> Result<Self> {
        let image_view = vk::image::view::ImageView::new_default(image.clone())?;
        let fb = vk::render_pass::Framebuffer::new(
            render_pass,
            vk::render_pass::FramebufferCreateInfo {
                attachments: vec![image_view],
                ..Default::default()
            },
        )?;
        Ok(Self {
            image,
            framebuffer: fb,
            command_buffer: todo!(),
        })
    }
}

pub struct Framebuffers(pub Vec<Framebuffer>);

impl Framebuffers {
    pub fn new(vkenv: &VulkanEnvironment, images: Vec<Arc<vk::image::swapchain::SwapchainImage>>, render_pass: &Arc<vk::render_pass::RenderPass>) -> Result<Self> {
        let framebuffers = images
            .into_iter()
            .map(|image| Framebuffer::new(render_pass.clone(), image))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(framebuffers))
    }
}

pub struct Swapchain {
    vkenv: Arc<VulkanEnvironment>,
    pub swapchain: Arc<vk::swapchain::Swapchain>,
    pub render_pass: Arc<vk::render_pass::RenderPass>,
    pub framebuffers: Framebuffers,
}

impl Swapchain {
    pub fn new(vkenv: &Arc<VulkanEnvironment>) -> Result<Self> {
        let caps = vkenv.physical_device
            .surface_capabilities(&vkenv.surface, Default::default())
            .map_err(|e| anyhow::anyhow!("failed to get surface capabilities from physical device: {}", e))?;
        let dimensions = vkenv.window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = Some(
                vkenv.physical_device
                    .surface_formats(&vkenv.surface, Default::default())
                    .map_err(|e| anyhow::anyhow!("failed to get surface formats from physical device: {}", e))?[0]
                    .0,
            );
        let (swapchain, images) = vk::swapchain::Swapchain::new(
                vkenv.device.clone(),
                vkenv.surface.clone(),
                vk::swapchain::SwapchainCreateInfo {
                    min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                    image_format,
                    image_extent: dimensions.into(),
                    image_usage: vk::image::ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                    composite_alpha,
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow::anyhow!("failed to create swapchain: {}", e))?;
        let render_pass = Self::new_render_pass(vkenv, &swapchain)?;
        let fbs = Framebuffers::new(vkenv, images, &render_pass)?;
        Ok(Self {
            vkenv: vkenv.clone(),
            swapchain,
            render_pass,
            framebuffers: fbs,
        })
    }
    pub fn recreate(&mut self, vkenv: &VulkanEnvironment, surface: &Arc<vk::swapchain::Surface>) -> Result<()> {
        let (swapchain, images) = self.swapchain.recreate(vk::swapchain::SwapchainCreateInfo {
                image_extent: vkenv.window.inner_size().into(),
                ..self.swapchain.create_info()
            })
            .map_err(|e| anyhow::anyhow!("failed to recreate swapchain: {}", e))?;
        self.swapchain = swapchain;
        self.framebuffers = Framebuffers::new(vkenv, images, &self.render_pass)?;
        Ok(())
    }
    fn new_render_pass(vkenv: &Arc<VulkanEnvironment>, swapchain: &Arc<vk::swapchain::Swapchain>) -> Result<Arc<vk::render_pass::RenderPass>> {
        vk::single_pass_renderpass!(
            vkenv.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(), // set the format the same as the swapchain
                    samples: 1,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .map_err(Into::into)
    }
}