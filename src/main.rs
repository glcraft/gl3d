// mod vulkan;
mod vkenv;
mod render;
use std::sync::Arc;

use vulkano as vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

// pub use vulkan::*;

fn main() {
    let event_loop = EventLoop::new(); // ignore this for now
                                       //
    let vkenv = Arc::new(vkenv::VulkanEnvironment::new(&event_loop).expect("failed to create environment"));

    let mut swapchain = render::Swapchain::new(vkenv.clone())
        .expect("failed to create swapchain");
    swapchain.framebuffers
        .build_command_buffer(|builder, render_begin_info| {
            builder
                .begin_render_pass(
                    vk::command_buffer::RenderPassBeginInfo {
                        clear_values: vec![Some([0.5, 0.1, 0.1, 1.0].into())],
                        ..render_begin_info
                    },
                    vk::command_buffer::SubpassContents::Inline,
                )
                .map_err(Into::<anyhow::Error>::into)?
                // .bind_pipeline_graphics(pipeline.clone())
                // .bind_vertex_buffers(0, vertex_buffer.clone())
                // .draw(vertex_buffer.len() as u32, 1, 0, 0)
                // .unwrap()
                .end_render_pass()
                .map_err(Into::<anyhow::Error>::into)?;
            Ok(())
        })
        .expect("failed to build command buffer");

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            swapchain.recreate();
        }
        Event::MainEventsCleared => {
            swapchain.draw().expect("failed to draw");
        }
        _ => (),
    });
}

