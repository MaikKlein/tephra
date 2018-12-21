pub mod vulkan;
use crate::buffer::BufferApi;
use crate::shader::ShaderApi;
//use renderpass::RenderpassApi;
//use pipeline::PipelineApi;
use crate::descriptor::{DescriptorApi, LayoutApi};
use crate::image::ImageApi;
use crate::render::RenderApi;
use crate::swapchain::SwapchainApi;
use crate::render::ComputeApi;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader: ShaderApi;
    type Buffer;
    type Render: RenderApi;
    type Compute: ComputeApi;
    type Image;
    type Swapchain: SwapchainApi;
    type Descriptor;
    type Layout: LayoutApi;
}
