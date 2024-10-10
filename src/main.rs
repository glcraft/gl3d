mod app;
mod engine;
mod error;

pub use crate::error::ApplicationError;

use winit::event_loop::{
    ControlFlow, 
    EventLoop
};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = app::App::default();
    event_loop.run_app(&mut app).map_err(|e| e.into())
}
