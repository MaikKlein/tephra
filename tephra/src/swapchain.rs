use backend::BackendApi;
use context::Context;
use image::Image;

pub trait CreateSwapchain {
    fn new(&self) -> Swapchain;
}

pub trait SwapchainApi {
    fn present_images(&self) -> &[Image];
    fn present(&self, index: u32);
    fn aquire_next_image(&self) -> u32;
}

pub struct Swapchain {
    pub data: Box<dyn SwapchainApi>,
}

impl Swapchain {
    pub fn new(ctx: &Context) -> Swapchain {
        CreateSwapchain::new(ctx.context.as_ref())
    }
    pub fn aquire_next_image(&self) -> u32 {
        self.data.aquire_next_image()
    }
    pub fn present_images(&self) -> &[Image] {
        self.data.present_images()
    }
    pub fn present(&self, index: u32){
        self.data.present(index)
    }
}
