use backend::BackendApi;
use context::Context;
use image::Image;

pub trait SwapchainApi {
    type Backend: BackendApi;
    fn new(context: &Context<Self::Backend>) -> Swapchain<Self::Backend>;
    fn present_images(&self) -> &[Image<Self::Backend>];
}

pub struct Swapchain<Backend: BackendApi> {
    pub data: Backend::Swapchain,
}

impl<Backend> Swapchain<Backend>
where
    Backend: BackendApi,
    Self: SwapchainApi<Backend = Backend>,
{
    pub fn new(context: &Context<Backend>) -> Swapchain<Backend> {
        <Self as SwapchainApi>::new(context)
    }
}
