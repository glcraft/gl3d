use vulkano as vk;
use std::sync::Arc;
use crate::vkenv::VulkanEnvironment;

type Result<T> = std::result::Result<T, anyhow::Error>;

struct Fence {
    into_boxed_closure: Option<Box<dyn FnOnce() -> Box<dyn vk::sync::GpuFuture>>>,
    wait_closure: Option<Box<dyn FnOnce() -> std::result::Result<(), vulkano::sync::FlushError>>>
}
impl Fence {
    fn get_boxed(&mut self) -> Box<dyn vk::sync::GpuFuture> {
        let opt = self.into_boxed_closure.take().map(|v| v());
        opt.unwrap()
    }
    fn wait(&mut self) -> std::result::Result<(), vulkano::sync::FlushError> {
        let opt = self.wait_closure.take().map(|v| v());
        opt.unwrap()
    }
}

pub struct Framebuffer {
    vkenv: Arc<VulkanEnvironment>,
    pub image: Arc<vk::image::swapchain::SwapchainImage>,
    pub framebuffer: Arc<vk::render_pass::Framebuffer>,
    pub command_buffer: Option<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>,
    fence: Option<Fence>
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
            fence: None
        })
    }
    pub fn build_command_buffer(&mut self, build_callback: &dyn Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()>,) -> Result<()> 
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
trait CommandBufferBuilder: Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()> + Clone + Sized
{}

impl<F> CommandBufferBuilder for F where F: Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()> + Clone 
{}

pub struct Framebuffers {
    list: Vec<Framebuffer>,
    cb_builder: Option<Arc<dyn Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()>>>
}

impl Framebuffers {
    pub fn new(vkenv: &Arc<VulkanEnvironment>, images: Vec<Arc<vk::image::swapchain::SwapchainImage>>, render_pass: &Arc<vk::render_pass::RenderPass>) -> Result<Self> {
        let framebuffers = images
            .into_iter()
            .map(|image| Framebuffer::new(vkenv.clone(), render_pass.clone(), image))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            list: framebuffers,
            cb_builder: None
        })
    }
    pub fn build_command_buffer<F>(&mut self, build_callback: F) -> Result<()> 
    where
        F: 'static+Fn(&mut AutoCommandBufferBuilder, vk::command_buffer::RenderPassBeginInfo) -> Result<()> + Clone,
    {
        for framebuffer in &mut self.list {
            framebuffer.build_command_buffer(&build_callback)?;
        }
        self.cb_builder = Some(Arc::new(build_callback));
        Ok(())
    }
    pub fn update_command_buffer(&mut self) -> Result<()> {
        if let Some(cb_builder) = &self.cb_builder {
            for framebuffer in &mut self.list {
                framebuffer.build_command_buffer(cb_builder.as_ref())?;
            }
        }
        Ok(())
    }
}

impl std::ops::Index<usize> for Framebuffers {
    type Output = Framebuffer;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
impl std::ops::IndexMut<usize> for Framebuffers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
    }
}
pub struct Swapchain {
    vkenv: Arc<VulkanEnvironment>,
    pub swapchain: Arc<vk::swapchain::Swapchain>,
    pub render_pass: Arc<vk::render_pass::RenderPass>,
    pub framebuffers: Framebuffers,
    previous_image_index: u32,
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
            previous_image_index: 0,
        })
    }
    pub fn recreate(&mut self) -> Result<()> {
        let (swapchain, images) = self.swapchain.recreate(vk::swapchain::SwapchainCreateInfo {
                image_extent: self.vkenv.dimension(),
                ..self.swapchain.create_info() 
            })
            .map_err(|e| anyhow::anyhow!("failed to recreate swapchain: {}", e))?;
        self.swapchain = swapchain;
        let cb_builder = self.framebuffers.cb_builder.clone();
        self.framebuffers = Framebuffers::new(&self.vkenv, images, &self.render_pass)?;
        self.framebuffers.cb_builder = cb_builder;
        self.framebuffers.update_command_buffer()?;
        Ok(())
    }
    pub fn draw(&mut self) -> Result<()> {
        use vulkano::sync::future::GpuFuture;
        use vulkano::sync::FlushError;
        let recreate_swapchain = 'a: {
            let mut recreate_swapchain = false;

            let previous_future = match &mut self.framebuffers[self.previous_image_index as usize].fence {
                // Create a `NowFuture`.
                None => {
                    let mut now = vk::sync::now(self.vkenv.device.clone());
                    now.cleanup_finished();
            
                    now.boxed()
                }
                // Use the existing `FenceSignalFuture`.
                Some(fence) => fence.get_boxed(),
            };
            
            let (image_i, suboptimal, acquire_future) =
                match vk::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(vk::swapchain::AcquireError::OutOfDate) => {
                        break 'a true;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };
            if suboptimal {
                recreate_swapchain = true;
            }
            if let Some(fence) = &mut self.framebuffers[image_i as usize].fence {
                fence.wait()?;
            }
            
            let queue = self.vkenv.queues.graphics.clone();
            let command_buffer = self.framebuffers[image_i as usize].command_buffer.clone().ok_or_else(|| anyhow::anyhow!("no command buffer"))?;
            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer)
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    vk::swapchain::SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
                )
                .then_signal_fence_and_flush();
            
            match future {
                Ok(future) => {
                    let fence1: Arc<vk::sync::future::FenceSignalFuture<_>> = Arc::new(future);
                    let fence2: Arc<vk::sync::future::FenceSignalFuture<_>> = Arc::clone(&fence1);
                    let new_fence = Fence {
                        into_boxed_closure: Some(Box::new(move || {
                            fence1.boxed()
                        })),
                        wait_closure: Some(Box::new(move || {
                            fence2.wait(None)
                        }))
                    };
                    
                    self.framebuffers[image_i as usize].fence = Some(new_fence);
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    self.framebuffers[image_i as usize].fence = None;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("failed to flush future: {e}"));
                }
            }
            self.previous_image_index = image_i;
            recreate_swapchain
        };
        if recreate_swapchain {
            self.recreate()?;
        }
        Ok(())
    }
}