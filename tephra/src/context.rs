use std::ops::Deref;
use std::sync::Arc;

use buffer::CreateBuffer;
use downcast;
use image::{CreateFramebuffer, CreateImage};
use pipeline::CreatePipeline;
use renderpass::CreateRenderpass;
use shader::CreateShader;
use swapchain::CreateSwapchain;

pub trait ContextApi: downcast::Downcast
where
    Self: CreateImage
        + CreateSwapchain
        + CreateShader
        + CreatePipeline
        + CreateRenderpass
        + CreateBuffer,
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
