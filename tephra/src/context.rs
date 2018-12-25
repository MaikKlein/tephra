use std::ops::Deref;
use std::sync::Arc;

use crate::{
    buffer::BufferApi,
    descriptor::{CreateLayout, CreatePool, DescriptorApi},
    downcast,
    image::ImageApi,
    pipeline::PipelineApi,
    renderpass::{FramebufferApi, RenderpassApi},
    shader::ShaderApi,
    swapchain::CreateSwapchain,
};

pub trait ContextApi: downcast::Downcast
where
    Self: CreateSwapchain
        + ShaderApi
        + DescriptorApi
        + CreateLayout
        + CreatePool
        + BufferApi
        + ImageApi
        + RenderpassApi
        + PipelineApi
        + FramebufferApi,
{
}
impl_downcast!(ContextApi);

#[derive(Clone)]
pub struct Context {
    pub context: Arc<dyn ContextApi>,
}

impl Deref for Context {
    type Target = ContextApi;
    fn deref(&self) -> &Self::Target {
        self.context.as_ref()
    }
}
