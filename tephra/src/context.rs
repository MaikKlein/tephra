use std::ops::Deref;
use std::sync::Arc;

use buffer::CreateBuffer;
use downcast;
use image::CreateImage;
// use pipeline::CreatePipeline;
// use renderpass::CreateRenderpass;
use descriptor::{CreatePool, CreateDescriptor, CreateLayout};
use render::{CreateCompute, CreateRender };
use shader::CreateShader;
use swapchain::CreateSwapchain;

pub trait ContextApi: downcast::Downcast
where
    Self: CreateImage
        + CreateSwapchain
        + CreateShader
        + CreateBuffer
        + CreateRender
        + CreateDescriptor
        + CreateLayout
        + CreatePool
        + CreateCompute
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
