use vulkano as vk;
use std::sync::Arc;
use crate::vkenv::VulkanEnvironment;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub struct Framebuffer {
    vkenv: Arc<VulkanEnvironment>,
    pub image: Arc<vk::image::swapchain::SwapchainImage>,
    pub framebuffer: Arc<vk::render_pass::Framebuffer>,
    pub command_buffer: Option<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>,
}

type AutoCommandBufferBuilder = vk::command_buffer::AutoCommandBufferBuilder<vk::command_buffer::PrimaryAutoCommandBuffer<<vk::command_buffer::allocator::StandardCommandBufferAllocator as vk::command_buffer::allocator::CommandBufferAllocator>::Alloc>, vk::command_buffer::allocator::StandardCommandBufferAllocator>;

impl Framebuffer {
    fn new(vkenv: Arc<VulkanEnvironment>, render_pass: Arc<vk::render_pass::RenderPass>, image: Arc<vk::image::swapchain::SwapchainImage>) -> Result<Self> {
        let image_view = vk::image::view::ImageView::new_default(image.clone())?;
        let fb = vk::render_pass::Framebuffer::new(
            render_pass,
            vk::render_pass::FramebufferCreateInfo {
                attachments: vec![image_view],
                ..Default::default()
            },
        )?;
        Ok(Self {
            vkenv,
            image,
            framebuffer: fb,
            command_buffer: None,
        })
    }
    pub fn build_command_buffer<F>(&mut self, build_callback: F) -> Result<()> 
    where
        F: Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()>,
    {
        let mut builder = vk::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vkenv.command_buffer_allocator,
            self.vkenv.queues.graphics.queue_family_index(),
            vk::command_buffer::CommandBufferUsage::MultipleSubmit, // don't forget to write the correct buffer usage
        )
        .map_err(Into::<anyhow::Error>::into)?;

        build_callback(&mut builder, vk::command_buffer::RenderPassBeginInfo::framebuffer(self.framebuffer.clone()))?;

        self.command_buffer = Some(Arc::new(builder.build().map_err(Into::<anyhow::Error>::into)?));

        Ok(())
    }
}

pub struct Framebuffers(Vec<Framebuffer>);

impl Framebuffers {
    pub fn new(vkenv: &Arc<VulkanEnvironment>, images: Vec<Arc<vk::image::swapchain::SwapchainImage>>, render_pass: &Arc<vk::render_pass::RenderPass>) -> Result<Self> {
        let framebuffers = images
            .into_iter()
            .map(|image| Framebuffer::new(vkenv.clone(), render_pass.clone(), image))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(framebuffers))
    }
    pub fn build_command_buffer<F>(&mut self, build_callback: F) -> Result<()> 
    where
        F: Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()> + Clone,
    {
        for framebuffer in &mut self.0 {
            framebuffer.build_command_buffer(build_callback.clone())?;
        }
        Ok(())
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