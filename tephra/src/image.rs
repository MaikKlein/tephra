use backend::BackendApi;
use context::Context;
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
pub trait ImageApi {
    type Backend: BackendApi;
    fn allocate(context: &Context<Self::Backend>, resolution: Resolution) -> Image<Self::Backend>;
    fn create_depth(
        context: &Context<Self::Backend>,
        resolution: Resolution,
    ) -> Image<Self::Backend>;
}

pub struct Image<Backend: BackendApi> {
    pub data: Backend::Image,
}
impl<Backend> Image<Backend>
where
    Backend: BackendApi,
    Self: ImageApi<Backend = Backend>,
{
    pub fn allocate(context: &Context<Backend>, resolution: Resolution) -> Image<Backend> {
        <Self as ImageApi>::allocate(context, resolution)
    }
    pub fn create_depth(context: &Context<Backend>, resolution: Resolution) -> Image<Backend> {
        <Self as ImageApi>::create_depth(context, resolution)
    }
}

pub struct RenderTargetInfo<'a, Backend: BackendApi> {
    pub image_views: Vec<&'a Image<Backend>>,
}

pub trait RenderTarget<Backend: BackendApi> {
    fn render_target(&self) -> RenderTargetInfo<Backend>;
}

pub trait FramebufferApi<Backend: BackendApi> {
    fn new<T: RenderTarget<Backend>>(context: &Context<Backend>) -> Framebuffer<T, Backend>;
}

pub struct Framebuffer<T, Backend>
where
    T: RenderTarget<Backend>,
    Backend: BackendApi,
{
    pub render_targets: T,
    pub data: Backend::Framebuffer,
}
