use std::ffi::CStr;

use crate::engine;
use ash::vk;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    pub window: Option<Window>,
    pub vk_engine: Option<engine::Engine>,
}

const VALIDATION_LAYER: &core::ffi::CStr = c"VK_LAYER_KHRONOS_validation";

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    fn instance_extensions() -> Vec<&'static CStr> {
        let mut instance_extentions = Vec::with_capacity(10);
        instance_extentions.push(ash::khr::surface::NAME);
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        instance_extentions.push(ash::khr::portability_enumeration::NAME);
        #[cfg(target_os = "macos")]
        instance_extentions.push(ash::ext::metal_surface::NAME);
        #[cfg(target_os = "ios")]
        instance_extentions.push(ash::mvk::ios_surface::NAME);
        #[cfg(target_os = "windows")]
        instance_extentions.push(ash::khr::win32_surface::NAME);
        #[cfg(all(target_os = "linux", feature = "wayland"))]
        instance_extentions.push(ash::khr::wayland_surface::NAME);
        #[cfg(all(target_os = "linux", not(feature = "wayland")))]
        instance_extentions.push(ash::khr::xlib_surface::NAME);

        instance_extentions
    }
    fn device_extensions() -> Vec<&'static CStr> {
        vec![
            ash::khr::swapchain::NAME,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME,
        ]
    }
    fn layers() -> Vec<&'static CStr> {
        vec![
            #[cfg(feature = "validation_layers")]
            VALIDATION_LAYER,
        ]
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        self.window = Some(window);
        let engine = engine::EngineBuilder::default()
            .app_info(engine::builder::ApplicationInfo {
                application_name: c"gly's app",
                application_version: 1,
                engine_name: c"gly's engine",
                engine_version: 1,
                api_version: vk::API_VERSION_1_3,
            })
            .instance_extensions(App::instance_extensions())
            .device_extensions(App::device_extensions())
            .layers(App::layers())
            .request_graphics_queue(engine::builder::QueueFamily {
                queue_count: 1,
                priority: 1.0,
            })
            .request_present_queue()
            .build_with_window(&self.window.as_ref().unwrap())
            .expect("failed to create engine");
        self.vk_engine = Some(engine); //Some(engine::Engine::new(unsafe { self.window.as_ref().unwrap_unchecked() }).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
