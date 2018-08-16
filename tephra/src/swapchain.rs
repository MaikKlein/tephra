use backend::BackendApi;
use context::Context;
use image::Image;

pub trait CreateSwapchain {
    fn new(&self) -> Swapchain;
}

pub trait SwapchainApi {
    fn present_images(&self) -> &[Image];
}

pub struct Swapchain {
    pub data: Box<dyn SwapchainApi>,
}

impl Swapchain {
    pub fn new(ctx: &Context) -> Swapchain {
        CreateSwapchain::new(ctx.context.as_ref())
    }
}
