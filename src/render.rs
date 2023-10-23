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
    pub fn new(vkenv: Arc<VulkanEnvironment>) -> Result<Self> {
        let caps = vkenv.surface_capabilities()?;
        let composite_alpha = caps.supported_composite_alpha
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no supported composite alpha"))?;
        let (swapchain, images) = vk::swapchain::Swapchain::new(
                vkenv.device.clone(),
                vkenv.surface.clone(),
                vk::swapchain::SwapchainCreateInfo {
                    min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                    image_format: vkenv.first_surface_format()?.map(|(f, _)| f),
                    image_extent: vkenv.dimension(),
                    image_usage: vk::image::ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                    composite_alpha,
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow::anyhow!("failed to create swapchain: {}", e))?;
        let render_pass = vkenv.new_render_pass(&swapchain)?;
        let framebuffers = Framebuffers::new(&vkenv, images, &render_pass)?;
        Ok(Self {
            vkenv,
            swapchain,
            render_pass,
            framebuffers,
        })
    }
    pub fn recreate(&mut self) -> Result<()> {
        let (swapchain, images) = self.swapchain.recreate(vk::swapchain::SwapchainCreateInfo {
                image_extent: self.vkenv.dimension(),
                ..self.swapchain.create_info()
            })
            .map_err(|e| anyhow::anyhow!("failed to recreate swapchain: {}", e))?;
        self.swapchain = swapchain;
        self.framebuffers = Framebuffers::new(&self.vkenv, images, &self.render_pass)?;
        Ok(())
    }
    
}