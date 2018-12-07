use buffer::{Buffer, BufferApi, BufferHandle};
use descriptor::{Allocator, Descriptor, DescriptorHandle, DescriptorInfo};
use framegraph::{Compiled, Framegraph, Resource, ResourceIndex};
use image::Image;
use pipeline::{ComputeState, PipelineState};
use render::RenderApi;
use renderpass::{VertexInput, VertexInputData};
use smallvec::SmallVec;
use std::marker::PhantomData;

// pub struct ShaderArgument;
// const MAX_SHADER_ARGS: usize = 4;
// pub type ShaderArguments = SmallVec<[ShaderArgument; MAX_SHADER_ARGS]>;

// // TODO: Implement properly
// pub struct PipelineInfo {
//     pub pipeline: PipelineState,
//     pub stride: u32,
//     pub vertex_input_data: Vec<VertexInputData>,
// }
// pub struct DrawCommand {
//     pub pipeline_info: PipelineInfo,
//     pub vertex: Resource<BufferHandle>,
//     pub index: Resource<Buffer<u32>>,
//     pub shader_arguments: DescriptorHandle,
// }

// pub struct DispatchCommand {
//     pub pipeline: ComputeState,
//     pub shader_arguments: DescriptorHandle,
//     pub x: u32,
//     pub y: u32,
//     pub z: u32,
// }

// pub struct CommandList {
//     commands: Vec<Command>,
// }

// impl CommandList {
//     pub fn dispatch<'alloc, ShaderArgument>(
//         &mut self,
//         pipeline: ComputeState,
//         descriptor: Descriptor<'alloc, ShaderArgument>,
//         x: u32,
//         y: u32,
//         z: u32,
//     ) where
//         ShaderArgument: DescriptorInfo,
//     {
//         let cmd = DispatchCommand {
//             pipeline,
//             shader_arguments: descriptor.handle,
//             x,
//             y,
//             z,
//         };
//         self.commands.push(Command::Dispatch(cmd));
//     }
//     pub fn draw_indexed<'alloc, Vertex, ShaderArgument>(
//         &mut self,
//         pipeline: PipelineState,
//         descriptor: Descriptor<'alloc, ShaderArgument>,
//         vertex_buffer: Resource<Buffer<Vertex>>,
//         index_buffer: Resource<Buffer<u32>>,
//     ) where
//         Vertex: VertexInput,
//         ShaderArgument: DescriptorInfo,
//     {
//         let pipeline_info = PipelineInfo {
//             pipeline,
//             stride: std::mem::size_of::<Vertex>() as u32,
//             vertex_input_data: Vertex::vertex_input_data(),
//         };
//         let cmd = DrawCommand {
//             pipeline_info,
//             shader_arguments: descriptor.handle,
//             vertex: vertex_buffer.buffer,
//             index: index_buffer,
//         };
//         self.commands.push(Command::Draw(cmd));
//     }
// }

// pub enum Command {
//     Draw(DrawCommand),
//     Dispatch(DispatchCommand),
// }

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
    BindDescriptor(DescriptorHandle),
}
pub struct ComputeCommandbuffer<'a> {
    fg: &'a Framegraph<Compiled>,
    pool_allocator: Allocator<'a>,
    pub(crate) cmds: Vec<ComputeCmd<'a>>,
}
impl<'a> ComputeCommandbuffer<'a> {
    pub fn new(pool_allocator: Allocator<'a>, fg: &'a Framegraph<Compiled>) -> Self {
        ComputeCommandbuffer {
            fg,
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
    pub fn bind_descriptor<T>(&mut self, descriptor: &T)
    where
        T: DescriptorInfo,
    {
        let mut d = self.pool_allocator.allocate::<T>();
        d.update(&self.fg.ctx, descriptor, &self.fg);
        let cmd = ComputeCmd::BindDescriptor(d.handle);
        self.cmds.push(cmd);
    }
}
pub enum GraphicsCmd<'a> {
    BindVertex(BufferHandle),
    BindIndex(BufferHandle),
    BindDescriptor(DescriptorHandle),
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
    pub fn bind_vertex<T>(&mut self, buffer: Buffer<T>) {
        let cmd = GraphicsCmd::BindVertex(buffer.buffer);
        self.cmds.push(cmd);
    }
    pub fn bind_index(&mut self, buffer: Buffer<u32>) {
        let cmd = GraphicsCmd::BindIndex(buffer.buffer);
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

    pub fn bind_descriptor<T>(&mut self, descriptor: &T)
    where
        T: DescriptorInfo,
    {
        let mut d = self.pool_allocator.allocate::<T>();
        d.update(&self.fg.ctx, descriptor, &self.fg);
        let cmd = GraphicsCmd::BindDescriptor(d.handle);
        self.cmds.push(cmd);
    }
}
