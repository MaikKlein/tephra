pub mod vulkan;
use crate::buffer::BufferApi;
//use renderpass::RenderpassApi;
//use pipeline::PipelineApi;
use crate::descriptor::{DescriptorApi, LayoutApi};
use crate::image::ImageApi;
use crate::swapchain::SwapchainApi;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader;
    type Buffer;
    type Image;
    type Swapchain: SwapchainApi;
    type Descriptor;
    type Layout: LayoutApi;
}
