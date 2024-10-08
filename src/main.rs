mod app;
mod engine;
mod error;

pub use crate::error::ApplicationError;

use winit::event_loop::{
        ControlFlow, 
        EventLoop
    };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = engine::Engine::new()?;
    println!("Physical Device: {:#?}", engine.physical_device);
    // println!("Logical Device: {:}", engine.logical_device);
    println!("Graphics Queue: {:#?}", engine.queues.graphics);
    Ok(())

    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    
    // let mut app = app::App::default();
    // event_loop.run_app(&mut app).map_err(|e| e.into())
}
