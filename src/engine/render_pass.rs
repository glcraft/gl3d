use std::{marker::PhantomData, ops::Deref};

use ash::{vk, Device};

pub struct RenderPass<'a> {
    render_pass: vk::RenderPass,
    device: &'a Device,
}

impl Deref for RenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe { self.device.destroy_render_pass(self.render_pass, None) };
    }
}

pub struct RenderPassBuilder<'a> {
    device: &'a Device,
    flags: vk::RenderPassCreateFlags,
    attachments: Vec<vk::AttachmentDescription>,
    subpasses: Vec<SubpassBuilder>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn build(self) -> Result<vk::RenderPass> {
        todo!()
    }
    pub fn set_attachments(mut self, value: Vec<vk::AttachmentDescription>) -> Self {
        self.attachments = value;
        self
    }
    pub fn add_attachment(mut self, value: vk::AttachmentDescription) -> Self {
        self.attachments.push(value);
        self
    }
    pub fn set_subpasses(mut self, value: Vec<SubpassBuilder<'a>>) -> Self {
        self.subpasses = value;
        self
    }
    pub fn add_subpass(mut self, value: SubpassBuilder) -> Self {
        self.subpasses.push(value);
        self
    }
    pub fn set_dependencies(mut self, value: Vec<vk::SubpassDependency>) -> Self {
        self.dependencies = value;
        self
    }
    pub fn add_dependency(mut self, value: vk::SubpassDependency) -> Self {
        self.dependencies.push(value);
        self
    }
    pub fn build(self) -> Result<RenderPass, vk::Result> {
        let subpasses = self
            .subpasses
            .iter()
            .map(SubpassBuilder::make_description)
            .collect::<Vec<_>>();
        let create_info = vk::RenderPassCreateInfo {
            flags: self.flags,
            attachment_count: self.attachments.as_ref().len() as _,
            p_attachments: self.attachments.as_ref().as_ptr(),
            subpass_count: subpasses.len() as _,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: self.dependencies.as_ref().len() as _,
            p_dependencies: self.dependencies.as_ref().as_ptr(),
            ..Default::default()
        };
        let render_pass = unsafe { self.device.create_render_pass(&create_info, None)? };
        Ok(RenderPass {
            render_pass,
            device: &self.device,
        })
    }
}

pub struct SubpassBuilder {
    flags: vk::SubpassDescriptionFlags,
    bind_point: vk::PipelineBindPoint,
    input_attachments: Vec<vk::AttachmentReference>,
    color_attachments: Vec<vk::AttachmentReference>,
    resolve_attachments: Vec<vk::AttachmentReference>,
    depth_stencil_attachment: Vec<vk::AttachmentReference>,
    preserve_attachments: Vec<u32>,
}

impl<'a, I, C, R, D, P> SubpassBuilder<'a, I, C, R, D, P> {
    pub fn set_input_attachments(mut self, value: Vec<vk::AttachmentReference>) -> Self {
        self.input_attachments = value;
        self
    }
    pub fn add_input_attachment(mut self, input: vk::AttachmentReference) -> Self {
        self.input_attachments.push(input);
        self
    }
    pub fn set_color_attachments(mut self, value: Vec<vk::AttachmentReference>) -> Self {
        self.color_attachments = value;
        self
    }
    pub fn add_color_attachment(mut self, color: vk::AttachmentReference) -> Self {
        self.color_attachments.push(color);
        self
    }
    pub fn set_resolve_attachments(mut self, value: Vec<vk::AttachmentReference>) -> Self {
        self.resolve_attachments = value;
        self
    }
    pub fn add_resolve_attachment(mut self, resolve: vk::AttachmentReference) -> Self {
        self.resolve_attachments.push(resolve);
        self
    }
    pub fn set_depth_stencil_attachments(mut self, value: Vec<vk::AttachmentReference>) -> Self {
        self.depth_stencil_attachments = value;
        self
    }
    pub fn add_depth_stencil_attachment(mut self, depth_stencil: vk::AttachmentReference) -> Self {
        self.depth_stencil_attachments.push(depth_stencil);
        self
    }
    pub fn set_preserve_attachments(mut self, value: Vec<u32>) -> Self {
        self.preserve_attachments = value;
        self
    }
    pub fn add_preserve_attachment(mut self, preserve: u32) -> Self {
        self.preserve_attachments.push(preserve);
        self
    }
    pub fn make_description(&self) -> vk::SubpassDescription {
        vk::SubpassDescription {
            flags: self.flags,
            input_attachment_count: self.input_attachments.as_ref().len() as _,
            p_input_attachments: self.input_attachments.as_ref().as_ptr(),
            color_attachment_count: self.color_attachments.as_ref().len() as _,
            p_color_attachments: self.color_attachments.as_ref().as_ptr(),
            p_resolve_attachments: self.resolve_attachments.as_ref().as_ptr(),
            p_depth_stencil_attachment: self.depth_stencil_attachment.as_ref().as_ptr(),
            preserve_attachment_count: self.preserve_attachments.as_ref().len() as _,
            p_preserve_attachments: self.preserve_attachments.as_ref().as_ptr(),
            ..Default::default()
        }
    }
}
