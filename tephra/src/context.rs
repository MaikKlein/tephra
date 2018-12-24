use std::ops::Deref;
use std::sync::Arc;

use crate::{
    buffer::BufferApi,
    descriptor::{CreateLayout, CreatePool, DescriptorApi},
    downcast,
    image::ImageApi,
    pipeline::PipelineApi,
    render::{CreateCompute, CreateRender},
    renderpass::RenderTargetApi,
    shader::CreateShader,
    swapchain::CreateSwapchain,
};

pub trait ContextApi: downcast::Downcast
where
    Self: CreateSwapchain
        + CreateShader
        + CreateRender
        + DescriptorApi
        + CreateLayout
        + CreatePool
        + CreateCompute
        + BufferApi
        + ImageApi
        + RenderTargetApi
        + PipelineApi,
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
