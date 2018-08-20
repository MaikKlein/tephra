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
    fn allocate(&self, desc: ImageDesc) -> Image;
}

pub trait ImageApi: Downcast {
    fn desc(&self) -> &ImageDesc;
}

impl_downcast!(ImageApi);

pub struct Image {
    pub data: Box<dyn ImageApi>,
}

pub struct ImageDesc {
    pub resolution: Resolution,
    pub layout: ImageLayout,
}

impl Image where {
    pub fn downcast<B: BackendApi>(&self) -> &B::Image {
        self.data
            .downcast_ref::<B::Image>()
            .expect("Downcast Image Vulkan")
    }
    pub fn allocate(ctx: &Context, desc: ImageDesc) -> Image {
        CreateImage::allocate(ctx.context.as_ref(), desc)
    }
}

pub struct RenderTargetInfo<'a> {
    pub image_views: Vec<&'a Image>,
}

pub trait RenderTarget<'a> {
    fn render_target(&self) -> RenderTargetInfo;
}

pub trait CreateFramebuffer {
    fn new(&self, render_target: &RenderTargetInfo) -> Self;
}

pub trait FramebufferApi: Downcast {}
impl_downcast!(FramebufferApi);

pub struct Framebuffer<T>
where
    for<'a> T: RenderTarget<'a>,
{
    pub render_target: T,
    pub data: Box<dyn FramebufferApi>,
}

// impl<Target> Framebuffer<Target>
// where
//     for<'a> Target: RenderTarget<'a>,
// {
//     pub fn new<P: Pass<'a>>(
//         context: &Context<Backend>,
//         target: P::Target,
//         renderpass: &Renderpass<P, Backend>,
//     ) -> Framebuffer<P::Target, Backend> {
//         <Self as FramebufferApi>::new(context, target, renderpass)
//     }
// }
