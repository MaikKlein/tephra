pub mod vulkan;
use shader::ShaderApi;
use buffer::BufferApi;
//use renderpass::RenderpassApi;
//use pipeline::PipelineApi;
use image::{ImageApi};
use swapchain::SwapchainApi;
use render::RenderApi;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader: ShaderApi;
    type Buffer: BufferApi;
    type Render: RenderApi;
    type Image: ImageApi;
    type Swapchain: SwapchainApi;
}
