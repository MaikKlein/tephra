pub mod vulkan;

pub trait BackendApi
where
    Self: Copy + Clone + Sized + 'static,
{
    type Shader;
    type Buffer;
    type Context: Clone;
    type Renderpass;
}
