use buffer::{Buffer, BufferApi, GenericBuffer};
use descriptor::{Allocator, Descriptor, DescriptorInfo, NativeDescriptor};
use framegraph::{Resource, ResourceIndex};
use image::Image;
use pipeline::PipelineState;
use render::RenderApi;
use renderpass::{VertexInput, VertexInputData};
use std::marker::PhantomData;
pub struct Graphics;
pub struct Compute;
// pub trait CreateCommandbuffer<Type> {
//     fn create_commandbuffer(&self) -> Commandbuffer<Type>;
// }
pub trait ExecuteApi {
    fn execute_commands(&self, cmds: &[GraphicsCmd]);
}
pub trait CreateExecute {
    fn create_execute(&self) -> Execute;
}

pub struct Execute {
    pub inner: Box<dyn ExecuteApi>,
}

pub enum GraphicsCmd<'a> {
    BindVertex(&'a GenericBuffer),
    BindIndex(&'a GenericBuffer),
    BindDescriptor(NativeDescriptor),
    BindPipeline {
        state: &'a PipelineState,
        stride: u32,
        vertex_input_data: Vec<VertexInputData>,
    },
    DrawIndex {
        len: u32,
    },
}

pub struct GraphicsCommandbuffer<'a> {
    pool_allocator: Allocator<'a>,
    pub(crate) cmds: Vec<GraphicsCmd<'a>>,
}

impl<'a> GraphicsCommandbuffer<'a> {
    pub fn new(pool_allocator: Allocator<'a>) -> Self {
        GraphicsCommandbuffer {
            cmds: Vec::new(),
            pool_allocator,
        }
    }
    pub fn bind_vertex<T>(&mut self, buffer: &'a Buffer<T>) {
        let cmd = GraphicsCmd::BindVertex(&buffer.buffer);
        self.cmds.push(cmd);
    }
    pub fn bind_index(&mut self, buffer: &'a Buffer<u32>) {
        let cmd = GraphicsCmd::BindIndex(&buffer.buffer);
        self.cmds.push(cmd);
    }
    pub fn bind_pipeline<T: VertexInput>(&mut self, state: &'a PipelineState) {
        let cmd = GraphicsCmd::BindPipeline {
            state,
            stride: std::mem::size_of::<T>() as u32,
            vertex_input_data: T::vertex_input_data(),
        };
        self.cmds.push(cmd);
    }
    pub fn draw_index(&mut self, len: usize) {
        let cmd = GraphicsCmd::DrawIndex { len: len as u32 };
        self.cmds.push(cmd);
    }

    pub fn bind_descriptor<T>(&mut self, descriptor: &'a T)
    where
        T: DescriptorInfo,
    {
        let mut d = self.pool_allocator.allocate::<T>();
        d.update(descriptor);
        let cmd = GraphicsCmd::BindDescriptor(d.inner_descriptor);
        self.cmds.push(cmd);
    }
}
