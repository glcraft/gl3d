use vulkano as vk;
use std::sync::Arc;
use crate::vkenv::VulkanEnvironment;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub struct Renderer {
    render_pass: Arc<vk::render_pass::RenderPass>,
    framebuffers: Vec<Arc<vk::render_pass::Framebuffer>>,
    pipeline: Arc<vk::pipeline::GraphicsPipeline>,
    command_buffers: Vec<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>,
}

impl Renderer {
    pub fn new(vkenv: &VulkanEnvironment) -> Result<Self> {
        let render_pass = Self::get_render_pass(vkenv)?;
        let framebuffers = Self::get_framebuffers(vkenv, &render_pass)?;
        let mut viewport = vk::pipeline::graphics::viewport::Viewport {
            origin: [0.0, 0.0],
            dimensions: vkenv.window.inner_size().into(),
            depth_range: 0.0..1.0,
        };
    
        let pipeline = Self::get_pipeline(vkenv, &render_pass, viewport)?;
        let command_buffers = Self::get_command_buffers(vkenv, &pipeline, &framebuffers)?;
        Ok(Self {
            render_pass,
            framebuffers,
            pipeline,
            command_buffers,
        })
    }
    fn get_render_pass(vkenv: &VulkanEnvironment) -> Result<Arc<vk::render_pass::RenderPass>> {
        vk::single_pass_renderpass!(
            vkenv.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: vkenv.swapchain.image_format(), // set the format the same as the swapchain
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
    fn get_framebuffers(
        vkenv: &VulkanEnvironment,
        render_pass: &Arc<vk::render_pass::RenderPass>,
    ) -> Result<Vec<Arc<vk::render_pass::Framebuffer>>> {
        vkenv.images
            .iter()
            .map(|image| {
                let view = vk::image::view::ImageView::new_default(image.clone())
                    .map_err(Into::<anyhow::Error>::into)?;
                vk::render_pass::Framebuffer::new(
                    render_pass.clone(),
                    vk::render_pass::FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .map_err(Into::into)
            })
            .collect::<Result<Vec<_>>>()
    }
    fn get_pipeline(
        vkenv: &VulkanEnvironment,
        render_pass: &Arc<vk::render_pass::RenderPass>,
        viewport: vk::pipeline::graphics::viewport::Viewport,
    ) -> Result<Arc<vk::pipeline::graphics::GraphicsPipeline>> {
        use vk::pipeline::graphics::input_assembly::InputAssemblyState;
        use vk::pipeline::graphics::viewport::{Viewport, ViewportState};
        use vk::pipeline::GraphicsPipeline;
        use vk::render_pass::Subpass;
        use vk::shader::ShaderModule;

        vk::pipeline::graphics::GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(vkenv.device.clone())
            .map_err(Into::into)
    }
    fn get_command_buffers(
        vkenv: &VulkanEnvironment,
        pipeline: &Arc<vk::pipeline::GraphicsPipeline>,
        framebuffers: &[Arc<vk::render_pass::Framebuffer>],
    ) -> Result<Vec<Arc<vk::command_buffer::PrimaryAutoCommandBuffer>>> {
        framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = vk::command_buffer::AutoCommandBufferBuilder::primary(
                    &vkenv.command_buffer_allocator,
                    vkenv.queues.graphics.queue_family_index(),
                    vk::command_buffer::CommandBufferUsage::MultipleSubmit, // don't forget to write the correct buffer usage
                )
                .map_err(Into::<anyhow::Error>::into)?;
    
                builder
                    .begin_render_pass(
                        vk::command_buffer::RenderPassBeginInfo {
                            clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                            ..vk::command_buffer::RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        vk::command_buffer::SubpassContents::Inline,
                    )
                    .map_err(Into::<anyhow::Error>::into)?
                    .bind_pipeline_graphics(pipeline.clone())
                    // .bind_vertex_buffers(0, vertex_buffer.clone())
                    // .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    // .unwrap()
                    .end_render_pass()
                    .map_err(Into::<anyhow::Error>::into)?;
    
                Ok(Arc::new(builder.build().map_err(Into::<anyhow::Error>::into)?))
            })
            .collect::<Result<Vec<_>>>()
    }
    
}