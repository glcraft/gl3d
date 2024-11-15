use ash::vk;
use core::ffi::CStr;

use crate::ApplicationError;

use super::Engine;

pub struct Builder {
    pub(super) instance_extensions: Vec<&'static CStr>,
    pub(super) device_extensions: Vec<&'static CStr>,
    pub(super) app_info: Option<ApplicationInfo>,
    pub(super) extent: Option<vk::Extent2D>,
    // window: Option<&winit::window::Window>,
    pub(super) queue_families: QueueFamilies,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            instance_extensions: Vec::new(),
            device_extensions: Vec::new(),
            app_info: None,
            extent: None,
            // window: None,
            queue_families: Default::default(),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn instance_extensions(mut self, extensions: Vec<&'static CStr>) -> Self {
        self.instance_extensions = extensions;
        self
    }
    pub fn device_extensions(mut self, extensions: Vec<&'static CStr>) -> Self {
        self.device_extensions = extensions;
        self
    }
    pub fn app_info(mut self, app_info: ApplicationInfo) -> Self {
        self.app_info = Some(app_info);
        self
    }
    pub fn request_graphics_queue(mut self, graphic_queue: QueueFamily) -> Self {
        self.queue_families.graphics = Some(graphic_queue);
        self
    }
    pub fn request_compute_queue(mut self, compute_queue: QueueFamily) -> Self {
        self.queue_families.compute = Some(compute_queue);
        self
    }
    pub fn request_present_queue(mut self) -> Self {
        self.queue_families.present = true;
        self
    }
    pub fn build(self) -> Result<Engine, ApplicationError> {
        Engine::new(self)
    }
    pub fn build_with_window(
        self,
        window: &winit::window::Window,
    ) -> Result<Engine, ApplicationError> {
        let builder = Builder {
            extent: Some(vk::Extent2D{
                width: window.inner_size().width as u32,
                height: window.inner_size().height as u32
            }),
            ..self
        };
        Engine::with_window(builder, window)
    }
}
fn default_device_support(_instance: &ash::Instance, _device: &ash::vk::PhysicalDevice) -> bool {
    true
}
#[derive(Default, Clone, Debug)]
pub struct ApplicationInfo {
    pub application_name: &'static CStr,
    pub application_version: u32,
    pub engine_name: &'static CStr,
    pub engine_version: u32,
    pub api_version: u32,
}

impl From<ApplicationInfo> for vk::ApplicationInfo<'_> {
    fn from(app_info: ApplicationInfo) -> Self {
        vk::ApplicationInfo {
            p_application_name: app_info.application_name.as_ptr(),
            application_version: app_info.application_version,
            p_engine_name: app_info.engine_name.as_ptr(),
            engine_version: app_info.engine_version,
            api_version: app_info.api_version,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug)]
#[non_exhaustive]
pub struct QueueFamilies {
    pub graphics: Option<QueueFamily>,
    pub compute: Option<QueueFamily>,
    pub present: bool,
}
#[derive(Clone, Debug)]
pub struct QueueFamily {
    pub queue_count: u32,
    pub priority: f32,
}
