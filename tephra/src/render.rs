use framegraph::{Framegraph, Compiled};
use backend::BackendApi;
use buffer::{Buffer, BufferApi};
use commandbuffer::GraphicsCmd;
use context::Context;
use downcast::Downcast;
use image::{Image, Resolution};
use pipeline::PipelineState;
use renderpass::{VertexInput, VertexInputData};
use std::mem::size_of;
use std::ops::Deref;
pub trait CreateRender {
    fn create_render(&self, resolution: Resolution, images: &[&Image]) -> Render;
}

pub trait RenderApi: Downcast {
    fn draw_indexed(
        &self,
        state: &PipelineState,
        stride: u32,
        vertex_input: &[VertexInputData],
        vertex_buffer: &BufferApi,
        index_buffer: &BufferApi,
        len: u32,
    );
    fn execute_commands(&self,fg: &Framegraph<Compiled>, cmds: &[GraphicsCmd]);
}
impl_downcast!(RenderApi);

impl Render {
    pub fn draw_indexed<I, D>(
        &self,
        state: &PipelineState,
        vertex_buffer: &Buffer<I>,
        index_buffer: &Buffer<u32>,
        descriptors: &[D],
    ) where
        I: VertexInput,
    {
        self.inner.draw_indexed(
            state,
            size_of::<I>() as u32,
            &I::vertex_input_data(),
            vertex_buffer.buffer.as_ref(),
            index_buffer.buffer.as_ref(),
            index_buffer.len(),
        );
    }
}
pub struct Render {
    pub inner: Box<dyn RenderApi>,
}
impl Deref for Render {
    type Target = RenderApi;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl RenderApi {
    pub fn downcast<B: BackendApi>(&self) -> &B::Render {
        self.downcast_ref::<B::Render>()
            .expect("Downcast Render Vulkan")
    }
}
impl Render {
    pub fn new(ctx: &Context, resolution: Resolution, images: &[&Image]) -> Render {
        ctx.create_render(resolution, images)
    }
}
