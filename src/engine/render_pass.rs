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

pub struct RenderPassBuilder<'a, AttachmentContainer, SubpassContainer, DependencyContainer> {
    device: &'a Device,
    flags: vk::RenderPassCreateFlags,
    attachments: AttachmentContainer,
    subpasses: SubpassContainer,
    dependencies: DependencyContainer,
}

impl<'a, A, S, D> RenderPassBuilder<'a, A, S, D> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn build(self) -> Result<vk::RenderPass> {
        todo!()
    }
    pub fn set_attachments(mut self, value: A) -> Self {
        self.attachments = value;
        self
    }
    pub fn set_subpasses(mut self, value: S) -> Self {
        self.subpasses = value;
        self
    }
    pub fn set_dependencies(mut self, value: D) -> Self {
        self.dependencies = value;
        self
    }
}
impl<'a, S, D> RenderPassBuilder<'a, Vec<vk::AttachmentDescription>, S, D> {
    pub fn add_attachment(mut self, value: vk::AttachmentDescription) -> Self {
        self.attachments.push(value);
        self
    }
}
impl<'a, A, D, I, C, R, SuD, P>
    RenderPassBuilder<'a, A, Vec<SubpassBuilder<'a, I, C, R, SuD, P>>, D>
{
    pub fn add_subpass(mut self, value: SubpassBuilder<'_, _, _, _, _, _>) -> Self {
        self.subpasses.push(value);
        self
    }
}
impl<'a, A, S> RenderPassBuilder<'a, A, S, Vec<vk::SubpassDependency>> {
    pub fn add_dependency(mut self, value: vk::SubpassDependency) -> Self {
        self.dependencies.push(value);
        self
    }
}
impl<'a, A, S, D> RenderPassBuilder<'a, A, S, D>
where
    A: AsRef<[vk::AttachmentDescription]>,
    S: AsRef<[SubpassBuilder<'a>]>,
    D: AsRef<[vk::SubpassDependency]>,
{
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

pub struct SubpassBuilder<
    'a,
    InputContainer,
    ColorContainer,
    ResolveContainer,
    DepthStencilContainer,
    PreserveContainer,
> {
    flags: vk::SubpassDescriptionFlags,
    bind_point: vk::PipelineBindPoint,
    input_attachments: InputContainer,
    color_attachments: ColorContainer,
    resolve_attachments: ResolveContainer,
    depth_stencil_attachment: DepthStencilContainer,
    preserve_attachments: PreserveContainer,
    _marker: PhantomData<&'a ()>,
}

impl<'a, I, C, R, D, P> SubpassBuilder<'a, I, C, R, D, P> {
    pub fn set_input_attachments(mut self, value: I) -> Self {
        self.input_attachments = value;
        self
    }
    pub fn set_color_attachments(mut self, value: C) -> Self {
        self.color_attachments = value;
        self
    }
    pub fn set_resolve_attachments(mut self, value: R) -> Self {
        self.resolve_attachments = value;
        self
    }
    pub fn set_depth_stencil_attachments(mut self, value: D) -> Self {
        self.depth_stencil_attachments = value;
        self
    }
    pub fn set_preserve_attachments(mut self, value: P) -> Self {
        self.preserve_attachments = value;
        self
    }
}
impl<'a, I, C, R, D, P> SubpassBuilder<'a, I, C, R, D, P>
where
    I: AsRef<[vk::AttachmentReference]>,
    C: AsRef<[vk::AttachmentReference]>,
    R: AsRef<[vk::AttachmentReference]>,
    D: AsRef<[vk::AttachmentReference]>,
    P: AsRef<[u32]>,
{
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
impl<'a, C, R, D, P> SubpassBuilder<'a, Vec<vk::AttachmentReference>, C, R, D, P> {
    pub fn add_input_attachment(mut self, input: vk::AttachmentReference) -> Self {
        self.input_attachments.push(input);
        self
    }
}
impl<'a, I, R, D, P> SubpassBuilder<'a, I, Vec<vk::AttachmentReference>, R, D, P> {
    pub fn add_color_attachment(mut self, color: vk::AttachmentReference) -> Self {
        self.color_attachments.push(color);
        self
    }
}
impl<'a, I, C, D, P> SubpassBuilder<'a, I, C, Vec<vk::AttachmentReference>, D, P> {
    pub fn add_resolve_attachment(mut self, resolve: vk::AttachmentReference) -> Self {
        self.resolve_attachments.push(resolve);
        self
    }
}
impl<'a, I, C, R, P> SubpassBuilder<'a, I, C, R, Vec<vk::AttachmentReference>, P> {
    pub fn add_depth_stencil_attachment(mut self, depth_stencil: vk::AttachmentReference) -> Self {
        self.depth_stencil_attachments.push(depth_stencil);
        self
    }
}

impl<'a, I, C, R, D> SubpassBuilder<'a, I, C, R, D, Vec<u32>> {
    pub fn add_preserve_attachment(mut self, preserve: u32) -> Self {
        self.preserve_attachments.push(preserve);
        self
    }
}
