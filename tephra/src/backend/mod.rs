pub mod vulkan;
use shader::ShaderApi;
use buffer::BufferApi;
use renderpass::RenderpassApi;
use pipeline::PipelineApi;
use image::{ImageApi, FramebufferApi};
use swapchain::SwapchainApi;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader: ShaderApi;
    type Buffer: BufferApi;
    type Renderpass: RenderpassApi;
    type Pipeline: PipelineApi;
    //type Render;
    type Framebuffer: FramebufferApi;
    type Image: ImageApi;
    type Swapchain: SwapchainApi;
}
