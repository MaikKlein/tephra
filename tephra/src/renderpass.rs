use backend::BackendApi;
use buffer::{Buffer, BufferProperty};
use context::Context;
use pipeline::Pipeline;
use std::marker::PhantomData;
use std::ops::Deref;
use ash::vk;

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
pub trait RenderApi<Backend: BackendApi> {
    fn new(context: &Context<Backend>) -> Self;
    fn draw_indexed<P, Vertex, Index>(
        &self,
        frame_buffer: vk::Framebuffer,
        renderpass: &Renderpass<P, Backend>,
        pipeline: Pipeline<Backend>,
        vertex: &Buffer<Vertex, impl BufferProperty, Backend>,
        index: &Buffer<Index, impl BufferProperty, Backend>,
    ) where
        P: Pass;
}

pub struct Render<Backend: BackendApi> {
    pub data: Backend::Render,
}

pub trait Pass {
    type Input: VertexInput;
    //fn render<Backend: BackendApi>(&self, render: &Render<Backend>) {}
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
    pub fn render(&self) {
        // let render = Render {
        // }
    }
}

// impl<P, Backend> Deref for Renderpass<P, Backend>
// where
//     P: Pass,
//     Backend: BackendApi,
// {
//     type Target = P;
//     fn deref(&self) -> &Self::Target {
//         &self.pass
//     }
// }

pub struct ImplRenderpass<Backend>
where
    Backend: BackendApi,
{
    pub data: Backend::Renderpass,
    pub _m: PhantomData<Backend>,
}
