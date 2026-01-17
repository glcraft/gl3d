use super::shader::Shader;
use ash::vk::{self, Pipeline};
use ash::Device;
use std::ops::Deref;

pub struct Builder<'a> {
    device: &'a Device,
    shaders: Vec<ShaderStageInfo<'a>>,
    input_assembly_state: Option<AssemblyState>,
    viewport_state: ViewportState,
    rasterizer_state: RasterizerState,
    multisampling_state: MultisamplingState,
    color_blend_state: ColorBlendState,
    dynamic_states: Vec<vk::DynamicState>,
}

impl<'a> Builder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            shaders: Vec::new(),
            input_assembly_state: None,
            viewport_state: Default::default(),
            rasterizer_state: Default::default(),
            multisampling_state: Default::default(),
            color_blend_state: Default::default(),
            dynamic_states: Vec::new(),
        }
    }
    pub fn build_graphics(self) -> Pipeline {
        let shader_stages = self
            .shaders
            .iter()
            .map(|info| vk::PipelineShaderStageCreateInfo {
                stage: info.kind,
                module: *info.module.deref(),
                p_name: unsafe { std::mem::transmute(info.entrypoint.as_ptr()) },
                // p_specialization_info: todo!(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: std::ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: std::ptr::null(),
            ..Default::default()
        };
        let input_assembly_state = self
            .input_assembly_state
            .map(
                |AssemblyState {
                     topology,
                     primitive_restart_enabled,
                 }| vk::PipelineInputAssemblyStateCreateInfo {
                    topology,
                    primitive_restart_enable: primitive_restart_enabled as _,
                    ..Default::default()
                },
            )
            .unwrap_or_default();
        let dynamic_states = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: self.dynamic_states.len() as _,
            p_dynamic_states: self.dynamic_states.as_ptr(),
            ..Default::default()
        };
        let color_blend_attachments = self
            .color_blend_state
            .attachments
            .into_iter()
            .map(|attachment| {
                let blend_enable = attachment.blend.is_some().into();
                let blend = attachment.blend.unwrap_or_default();
                vk::PipelineColorBlendAttachmentState {
                    blend_enable: blend_enable,
                    src_color_blend_factor: blend.color.src,
                    dst_color_blend_factor: blend.color.dst,
                    color_blend_op: blend.color.op,
                    src_alpha_blend_factor: blend.alpha.src,
                    dst_alpha_blend_factor: blend.alpha.dst,
                    alpha_blend_op: blend.alpha.op,
                    color_write_mask: attachment.mask,
                }
            })
            .collect::<Vec<_>>();
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: self.color_blend_state.logic_op.is_some().into(),
            logic_op: self.color_blend_state.logic_op.unwrap_or_default(),
            attachment_count: color_blend_attachments.len() as _,
            p_attachments: color_blend_attachments.as_ptr(),
            blend_constants: self.color_blend_state.blend_constant,
            ..Default::default()
        };
        // let color_blend
        let pipeline = vk::GraphicsPipelineCreateInfo {
            stage_count: todo!(),
            p_stages: todo!(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly_state,
            // p_tessellation_state: todo!(),
            p_viewport_state: &(&self.viewport_state).into(),
            p_rasterization_state: &self.rasterizer_state.into(),
            p_multisample_state: &self.multisampling_state.into(),
            p_depth_stencil_state: todo!(),
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: &dynamic_states,
            layout: todo!(),
            render_pass: todo!(),
            subpass: todo!(),
            base_pipeline_handle: todo!(),
            base_pipeline_index: todo!(),
            ..Default::default()
        };
        // self.device.create_graphics_pipelines(pipeline_cache, create_infos, allocation_callbacks)
        todo!()
    }
    pub fn add_shader<S: Into<String>>(
        mut self,
        module: Shader<'a>,
        kind: vk::ShaderStageFlags,
        entrypoint: S,
    ) -> Self {
        self.shaders.push(ShaderStageInfo {
            module,
            kind,
            entrypoint: entrypoint.into(),
        });
        self
    }
    pub fn set_input_assembly_state(
        mut self,
        topology: vk::PrimitiveTopology,
        primitive_restart_enabled: bool,
    ) -> Self {
        self.input_assembly_state = Some(AssemblyState {
            topology,
            primitive_restart_enabled,
        });
        self
    }
    pub fn add_viewport(mut self, viewport: vk::Viewport) -> Self {
        self.viewport_state.viewport.push(viewport);
        self
    }
    pub fn add_scissor(mut self, scissor: vk::Rect2D) -> Self {
        self.viewport_state.scissor.push(scissor);
        self
    }
    pub fn enable_depth_clamp(mut self) -> Self {
        self.rasterizer_state.depth_clamp_enable = true;
        self
    }
    pub fn enable_rasterizer_discard(mut self) -> Self {
        self.rasterizer_state.rasterizer_discard_enable = true;
        self
    }
    pub fn set_polygon_mode(mut self, polygon_mode: vk::PolygonMode) -> Self {
        self.rasterizer_state.polygon_mode = polygon_mode;
        self
    }
    pub fn set_line_width(mut self, line_width: f32) -> Self {
        self.rasterizer_state.line_width = line_width;
        self
    }
    pub fn set_cull_mode(
        mut self,
        cull_mode: vk::CullModeFlags,
        front_face: vk::FrontFace,
    ) -> Self {
        self.rasterizer_state.cull_mode = cull_mode;
        self.rasterizer_state.front_face = front_face;
        self
    }
    pub fn enable_depth_bias(
        mut self,
        constant_factor: f32,
        clamp: f32,
        slope_factor: f32,
    ) -> Self {
        self.rasterizer_state.depth_bias_enable = true;
        self.rasterizer_state.depth_bias_constant_factor = constant_factor;
        self.rasterizer_state.depth_bias_clamp = clamp;
        self.rasterizer_state.depth_bias_slope_factor = slope_factor;
        self
    }
    pub fn enable_color_blend_logic_op(mut self, logic_op: vk::LogicOp) -> Self {
        self.color_blend_state.logic_op = Some(logic_op);
        self
    }
    pub fn set_color_blend_constants(mut self, constants: [f32; 4]) -> Self {
        self.color_blend_state.blend_constant = constants;
        self
    }
    pub fn add_color_blend_attachment(mut self, attachment: ColorBlendAttachment) -> Self {
        self.color_blend_state.attachments.push(attachment);
        self
    }
    pub fn add_dynamic_state(mut self, state: vk::DynamicState) -> Self {
        self.dynamic_states.push(state);
        self
    }
    pub fn set_dynamic_states(mut self, states: Vec<vk::DynamicState>) -> Self {
        self.dynamic_states = states;
        self
    }
}
pub struct ShaderStageInfo<'a> {
    module: Shader<'a>,
    kind: vk::ShaderStageFlags,
    entrypoint: String,
}
pub struct AssemblyState {
    topology: vk::PrimitiveTopology,
    primitive_restart_enabled: bool,
}
pub struct ViewportState {
    viewport: Vec<vk::Viewport>,
    scissor: Vec<vk::Rect2D>,
}
impl<'a> From<&'a ViewportState> for vk::PipelineViewportStateCreateInfo<'a> {
    fn from(value: &'a ViewportState) -> Self {
        Self {
            viewport_count: value.viewport.len() as _,
            p_viewports: value.viewport.as_ptr(),
            scissor_count: value.scissor.len() as _,
            p_scissors: value.scissor.as_ptr(),
            ..Default::default()
        }
    }
}
impl Default for ViewportState {
    fn default() -> Self {
        Self {
            viewport: Vec::new(),
            scissor: Vec::new(),
        }
    }
}
pub struct RasterizerState {
    pub depth_clamp_enable: bool,
    pub rasterizer_discard_enable: bool,
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub front_face: vk::FrontFace,
    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}

impl<'a> From<RasterizerState> for vk::PipelineRasterizationStateCreateInfo<'a> {
    fn from(value: RasterizerState) -> Self {
        Self {
            depth_clamp_enable: value.depth_clamp_enable.into(),
            rasterizer_discard_enable: value.rasterizer_discard_enable.into(),
            polygon_mode: value.polygon_mode,
            cull_mode: value.cull_mode,
            front_face: value.front_face,
            depth_bias_enable: value.depth_bias_enable.into(),
            depth_bias_constant_factor: value.depth_bias_constant_factor,
            depth_bias_clamp: value.depth_bias_clamp,
            depth_bias_slope_factor: value.depth_bias_slope_factor,
            line_width: value.line_width,
            ..Default::default()
        }
    }
}
impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
        }
    }
}

pub struct MultisamplingState {
    sample_shading_enable: bool,
    rasterization_samples: vk::SampleCountFlags,
    min_sample_shading: f32,
    // p_sample_mask:
    alpha_to_coverage_enable: bool,
    alpha_to_one_enable: bool,
}

impl<'a> From<MultisamplingState> for vk::PipelineMultisampleStateCreateInfo<'a> {
    fn from(value: MultisamplingState) -> Self {
        Self {
            sample_shading_enable: value.sample_shading_enable.into(),
            rasterization_samples: value.rasterization_samples,
            min_sample_shading: value.min_sample_shading,
            // p_sample_mask:
            alpha_to_coverage_enable: value.alpha_to_coverage_enable.into(),
            alpha_to_one_enable: value.alpha_to_one_enable.into(),
            ..Default::default()
        }
    }
}

impl Default for MultisamplingState {
    fn default() -> Self {
        Self {
            sample_shading_enable: false,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            alpha_to_coverage_enable: false,
            alpha_to_one_enable: false,
        }
    }
}
pub struct ColorBlendState {
    logic_op: Option<vk::LogicOp>,
    attachments: Vec<ColorBlendAttachment>,
    blend_constant: [f32; 4],
}

impl Default for ColorBlendState {
    fn default() -> Self {
        Self {
            logic_op: None,
            attachments: Vec::new(),
            blend_constant: [0.0; 4],
        }
    }
}

pub struct ColorBlendAttachment {
    mask: vk::ColorComponentFlags,
    blend: Option<ColorAlphaBlend>,
}
impl ColorBlendAttachment {
    pub fn set_color_write_mask(mut self, mask: vk::ColorComponentFlags) -> Self {
        self.mask = mask;
        self
    }
    pub fn set_blend_mode(mut self, color: Blend, alpha: Blend) -> Self {
        self.blend = Some(ColorAlphaBlend { color, alpha });
        self
    }
}
pub struct ColorAlphaBlend {
    color: Blend,
    alpha: Blend,
}

impl Default for ColorAlphaBlend {
    fn default() -> Self {
        Self {
            color: Default::default(),
            alpha: Default::default(),
        }
    }
}
pub struct Blend {
    pub src: vk::BlendFactor,
    pub dst: vk::BlendFactor,
    pub op: vk::BlendOp,
}
impl Default for Blend {
    fn default() -> Self {
        Self {
            src: vk::BlendFactor::ONE,
            dst: vk::BlendFactor::ZERO,
            op: vk::BlendOp::ADD,
        }
    }
}
