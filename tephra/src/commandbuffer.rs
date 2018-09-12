use buffer::{Buffer, BufferApi, GenericBuffer};
use descriptor::{Allocator, Descriptor, DescriptorInfo, NativeDescriptor};
use framegraph::{Compiled, Framegraph, Resource, ResourceIndex};
use image::Image;
use pipeline::{ComputeState, PipelineState};
use render::RenderApi;
use renderpass::{VertexInput, VertexInputData};
use std::marker::PhantomData;
pub struct Graphics;
pub struct Compute;
pub trait ExecuteApi {
    fn execute_commands(&self, cmds: &[GraphicsCmd]);
}
pub trait CreateExecute {
    fn create_execute(&self) -> Execute;
}

pub struct Execute {
    pub inner: Box<dyn ExecuteApi>,
}

pub enum ComputeCmd<'a> {
    BindPipeline { state: &'a ComputeState },
    Dispatch { x: u32, y: u32, z: u32 },
    BindDescriptor(NativeDescriptor),
}
pub struct ComputeCommandbuffer<'a> {
    pool_allocator: Allocator<'a>,
    pub(crate) cmds: Vec<ComputeCmd<'a>>,
}
impl<'a> ComputeCommandbuffer<'a> {
    pub fn new(pool_allocator: Allocator<'a>) -> Self {
        ComputeCommandbuffer {
            cmds: Vec::new(),
            pool_allocator,
        }
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        let cmd = ComputeCmd::Dispatch { x, y, z };
        self.cmds.push(cmd);
    }

    pub fn bind_pipeline(&mut self, state: &'a ComputeState) {
        let cmd = ComputeCmd::BindPipeline { state };
        self.cmds.push(cmd);
    }
    pub fn bind_descriptor<T>(&mut self, descriptor: &'a T)
    where
        T: DescriptorInfo,
    {
        let mut d = self.pool_allocator.allocate::<T>();
        d.update(descriptor);
        let cmd = ComputeCmd::BindDescriptor(d.inner_descriptor);
        self.cmds.push(cmd);
    }
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

pub struct GraphicsCommandbuffer<'a, > {
    fg: &'a Framegraph<Compiled>,
    pool_allocator: Allocator<'a>,
    pub(crate) cmds: Vec<GraphicsCmd<'a>>,
}

impl<'a> GraphicsCommandbuffer<'a> {
    pub fn new(fg: &'a Framegraph<Compiled>, pool_allocator: Allocator<'a>) -> Self {
        GraphicsCommandbuffer {
            fg,
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
