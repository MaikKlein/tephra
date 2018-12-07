use context::Context;
use image::{Image, Resolution};
use std::ops::Deref;

#[derive(Debug, Fail)]
pub enum SwapchainError {
    #[fail(display = "Swapchain is out of date")]
    OutOfDate,
    #[fail(display = "Swapchain is Suboptimal")]
    Suboptimal,
    #[fail(display = "Unknown error")]
    Unknown,
}
pub trait CreateSwapchain {
    fn new(&self) -> Swapchain;
}

pub trait SwapchainApi {
    fn present_images(&self) -> &[Image];
    fn present(
        &self,
        index: u32,
    );
    fn aquire_next_image(&self) -> Result<u32, SwapchainError>;
    fn resolution(&self) -> Resolution;
    fn recreate(&mut self);
    fn copy_and_present(
        &self,
        image: Image,
    );
}

pub struct Swapchain {
    pub data: Box<dyn SwapchainApi>,
}

impl Deref for Swapchain {
    type Target = SwapchainApi;
    fn deref(&self) -> &Self::Target {
        self.data.as_ref()
    }
}

impl Swapchain {
    pub fn new(ctx: &Context) -> Swapchain {
        CreateSwapchain::new(ctx.context.as_ref())
    }

    pub fn recreate(&mut self) {
        use std::ops::DerefMut;
        SwapchainApi::recreate(self.data.deref_mut());
    }
}
