pub mod vulkan;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Context: Clone;
    type Shader;
    type Buffer;
    type Renderpass;
    type Pipeline;
    type Render;
    type Framebuffer;
    type Image;
    type Swapchain;
}
