mod vulkan;
mod vkenv;
mod render;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub use vulkan::*;

fn main() {
    let event_loop = EventLoop::new(); // ignore this for now
                                       //
    let vkenv = vkenv::VulkanEnvironment::new(&event_loop).expect("failed to create environment");

    let renderer = render::Renderer::new(&vkenv).expect("failed to create renderer");

    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => (),
    });
}

