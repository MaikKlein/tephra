use backend::BackendApi;
use context::Context;
use downcast::Downcast;
use renderpass::{Pass, Renderpass};
use std::any::Any;
pub enum ImageLayout {
    Undefined,
    Color,
    Depth,
}
#[derive(Debug, Copy, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

pub trait CreateImage {
    fn allocate(&self, resolution: Resolution) -> Image;
    fn create_depth(&self, resolution: Resolution) -> Image;
}

pub trait DowncastImage
where
    Self: ImageApi,
{
    fn downcast<B: BackendApi>(&self) -> &B::Image {
        self.as_any().downcast_ref::<B::Image>().expect("Downcast Image Vulkan")
    }
}
impl DowncastImage for ImageApi {}

pub trait ImageApi: Downcast {}

impl_downcast!(ImageApi);

pub struct Image {
    pub data: Box<dyn ImageApi>,
}

impl Image where {
    pub fn allocate(ctx: &Context, resolution: Resolution) -> Image {
        CreateImage::allocate(ctx.context.as_ref(), resolution)
    }

    pub fn create_depth(context: &Context, resolution: Resolution) -> Image {
        context.create_depth(resolution)
    }
}

pub struct RenderTargetInfo<'a> {
    pub image_views: Vec<&'a Image>,
}

pub trait RenderTarget {
    fn render_target(&self) -> RenderTargetInfo;
}

pub trait CreateFramebuffer {
    fn new(&self, render_target: &RenderTargetInfo) -> Self;
}

pub trait FramebufferApi: Downcast {}
impl_downcast!(FramebufferApi);

pub struct Framebuffer<T>
where
    T: RenderTarget,
{
    pub render_target: T,
    pub data: Box<dyn FramebufferApi>,
}

// impl<Target> Framebuffer<Target>
// where
//     Target: RenderTarget,
// {
//     pub fn new<T: RenderTarget, P: Pass>(
//         context: &Context<Backend>,
//         target: T,
//         renderpass: &Renderpass<P, Backend>,
//     ) -> Framebuffer<T, Backend> {
//         <Self as FramebufferApi>::new(context, target, renderpass)
//     }
// }
