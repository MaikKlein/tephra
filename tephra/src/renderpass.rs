use backend::BackendApi;
use context::Context;
use std::marker::PhantomData;

pub enum VertexType {
    F32(usize),
}

pub struct VertexInputData {
    pub vertex_type: VertexType,
    pub binding: usize,
    pub location: usize,
}
pub trait VertexInput {
    fn vertex_input_data() -> Vec<VertexInputData>;
}

pub trait Pass {
    type Input: VertexInput;
}

pub trait RenderpassApi<Backend>
where
    Backend: BackendApi,
{
    fn new(context: &Context<Backend>) -> Self;
}

pub struct Renderpass<P: Pass, Backend: BackendApi> {
    pub impl_render_pass: ImplRenderpass<Backend>,
    pub pass: P,
}
impl<P, Backend> Renderpass<P, Backend>
where
    P: Pass,
    Backend: BackendApi,
    ImplRenderpass<Backend>: RenderpassApi<Backend>,
{
    pub fn new(context: &Context<Backend>, pass: P) -> Self {
        let impl_render_pass = RenderpassApi::new(context);
        Renderpass {
            impl_render_pass,
            pass,
        }
    }
}

pub struct ImplRenderpass<Backend>
where
    Backend: BackendApi,
{
    pub data: Backend::Renderpass,
    pub _m: PhantomData<Backend>,
}

