use std::ops::Deref;
use std::sync::Arc;

use downcast;
use image::ImageApi;
// use pipeline::CreatePipeline;
// use renderpass::CreateRenderpass;
use buffer::BufferApi;
use descriptor::{CreateLayout, CreatePool, DescriptorApi};
use render::{CreateCompute, CreateRender};
use shader::CreateShader;
use swapchain::CreateSwapchain;

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
