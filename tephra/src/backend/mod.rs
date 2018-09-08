pub mod vulkan;
use buffer::BufferApi;
use shader::ShaderApi;
//use renderpass::RenderpassApi;
//use pipeline::PipelineApi;
use descriptor::{DescriptorApi, LayoutApi};
use image::ImageApi;
use render::RenderApi;
use swapchain::SwapchainApi;

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
    type Descriptor: DescriptorApi;
    type Layout: LayoutApi;
}
