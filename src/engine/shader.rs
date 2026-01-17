use ash::vk::Result as Error;
use ash::Device;

pub struct Shader<'a> {
    shader_module: ash::vk::ShaderModule,
    device: &'a Device,
}

impl<'a> Shader<'a> {
    pub fn from_spirv(device: &'a Device, data: Vec<u8>) -> Result<Self, Error> {
        let ptr = unsafe { std::mem::transmute(data.as_ptr()) };
        let create_info = ash::vk::ShaderModuleCreateInfo {
            code_size: data.len(),
            p_code: ptr,
            ..Default::default()
        };
        Ok(Self {
            shader_module: unsafe { device.create_shader_module(&create_info, None)? },
            device,
        })
    }
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_shader_module(self.shader_module, None) };
    }
}
