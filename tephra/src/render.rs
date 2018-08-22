use context::Context;
use buffer::{Buffer, BufferApi};
use framegraph::{ResourceMap, Compiled, Framegraph, Resource};
use image::Image;
use pipeline::PipelineState;
use renderpass::{VertexInput, VertexInputData};
use std::mem::size_of;
pub trait CreateRender {
    fn create_render(&self, images: &[&Image]) -> Render;
}

pub trait RenderApi {
    fn draw_indexed(
        &self,
        state: &PipelineState,
        stride: u32,
        vertex_input: &[VertexInputData],
        vertex_buffer: &BufferApi,
        index_buffer: &BufferApi,
        len: u32,
    );
}

impl Render {
    pub fn draw_indexed<I: VertexInput>(
        &self,
        state: &PipelineState,
        vertex_buffer: &Buffer<I>,
        index_buffer: &Buffer<u32>,
    ) {
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

impl Render {
    pub fn new(ctx: &Context, images: &[&Image]) -> Render {
        ctx.create_render(images)
    }
}
