use crate::{
    buffer::{Buffer, BufferApi, BufferHandle},
    descriptor::{Allocator, Descriptor, DescriptorHandle, DescriptorInfo},
    framegraph::{Compiled, Framegraph, Resource, ResourceIndex},
    image::{Image, ImageHandle},
    pipeline::{ComputePipeline, ComputePipelineState, GraphicsPipeline, GraphicsPipelineState},
    renderpass::{Framebuffer, Renderpass, VertexInput, VertexInputData},
};
use derive_builder::Builder;
use smallvec::SmallVec;
use std::{marker::PhantomData, ops::Range};

pub struct ShaderArgument;
const MAX_SHADER_ARGS: usize = 4;
pub type ShaderArguments = SmallVec<[ShaderArgument; MAX_SHADER_ARGS]>;

// TODO: Implement properly
pub struct PipelineInfo {
    pub pipeline: GraphicsPipelineState,
    pub stride: u32,
    pub vertex_input_data: Vec<VertexInputData>,
}
pub struct CopyImage {
    pub src: ImageHandle,
    pub dst: ImageHandle,
}

pub struct DrawCommand {
    pub graphics_pipeline: GraphicsPipeline,
    pub renderpass: Renderpass,
    pub framebuffer: Framebuffer,
    pub vertex: BufferHandle,
    pub index: Buffer<u32>,
    pub shader_arguments: DescriptorHandle,
    pub range: Range<u32>,
}

pub struct DispatchCommand {
    pub pipeline: ComputePipeline,
    pub shader_arguments: DescriptorHandle,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub enum QueueType {
    Graphics,
    Compute,
    Transfer,
}
pub trait GetQueueType {
    const TYPE: QueueType;
}
pub enum Graphics {}
impl GetQueueType for Graphics {
    const TYPE: QueueType = QueueType::Graphics;
}

pub enum Compute {}
impl GetQueueType for Compute {
    const TYPE: QueueType = QueueType::Compute;
}
pub enum Transfer {}
impl GetQueueType for Transfer {
    const TYPE: QueueType = QueueType::Transfer;
}
pub struct Submit {
    pub queue_ty: QueueType,
    pub commands: Vec<Command>,
}

pub struct CommandList {
    pub submits: Vec<Submit>,
}
impl CommandList {
    pub fn new() -> Self {
        CommandList {
            submits: Vec::new(),
        }
    }
    pub fn record<Q>(&mut self) -> RecordCommandList<Q> {
        RecordCommandList {
            command_list: self,
            commands: Vec::new(),
            _m: PhantomData,
        }
    }
}
pub struct RecordCommandList<'a, Q> {
    command_list: &'a mut CommandList,
    commands: Vec<Command>,
    _m: PhantomData<Q>,
}

impl<Q> RecordCommandList<'_, Q>
where
    Q: GetQueueType,
{
    pub fn submit(self) {
        let submit = Submit {
            queue_ty: Q::TYPE,
            commands: self.commands,
        };
        self.command_list.submits.push(submit);
    }
}
impl RecordCommandList<'_, Transfer> {}
impl RecordCommandList<'_, Graphics> {
    pub fn draw_indexed<'alloc, Vertex, ShaderArgument>(
        mut self,
        graphics_pipeline: GraphicsPipeline,
        renderpass: Renderpass,
        framebuffer: Framebuffer,
        descriptor: Descriptor<'alloc, ShaderArgument>,
        vertex_buffer: Buffer<Vertex>,
        index_buffer: Buffer<u32>,
        range: Range<u32>,
    ) -> Self
    where
        Vertex: VertexInput,
        ShaderArgument: DescriptorInfo,
    {
        let cmd = DrawCommand {
            graphics_pipeline,
            renderpass,
            framebuffer,
            shader_arguments: descriptor.handle,
            vertex: vertex_buffer.buffer,
            index: index_buffer,
            range,
        };
        self.commands.push(Command::Draw(cmd));
        self
    }
}
impl RecordCommandList<'_, Compute> {
    pub fn dispatch<'alloc, ShaderArgument>(
        mut self,
        pipeline: ComputePipeline,
        descriptor: Descriptor<'alloc, ShaderArgument>,
        x: u32,
        y: u32,
        z: u32,
    ) where
        ShaderArgument: DescriptorInfo,
    {
        let cmd = DispatchCommand {
            pipeline,
            shader_arguments: descriptor.handle,
            x,
            y,
            z,
        };
        self.commands.push(Command::Dispatch(cmd));
    }
}

pub enum Command {
    CopyImage(CopyImage),
    Draw(DrawCommand),
    Dispatch(DispatchCommand),
}
pub trait SubmitApi {
    unsafe fn submit_commands(&self, commands: &CommandList);
}

// pub trait ExecuteApi {
//     fn execute_commands(
//         &self,
//         cmds: &[GraphicsCmd],
//     );
// }
// pub trait CreateExecute {
//     fn create_execute(&self) -> Execute;
// }

// pub struct Execute {
//     pub inner: Box<dyn ExecuteApi>,
// }

// pub enum ComputeCmd<'a> {
//     BindPipeline { state: &'a ComputeState },
//     Dispatch { x: u32, y: u32, z: u32 },
//     BindDescriptor(DescriptorHandle),
// }
// pub struct ComputeCommandbuffer<'a> {
//     fg: &'a Framegraph<Compiled>,
//     pool_allocator: Allocator<'a>,
//     pub(crate) cmds: Vec<ComputeCmd<'a>>,
// }
// impl<'a> ComputeCommandbuffer<'a> {
//     pub fn new(
//         pool_allocator: Allocator<'a>,
//         fg: &'a Framegraph<Compiled>,
//     ) -> Self {
//         ComputeCommandbuffer {
//             fg,
//             cmds: Vec::new(),
//             pool_allocator,
//         }
//     }

//     pub fn dispatch(
//         &mut self,
//         x: u32,
//         y: u32,
//         z: u32,
//     ) {
//         let cmd = ComputeCmd::Dispatch { x, y, z };
//         self.cmds.push(cmd);
//     }

//     pub fn bind_pipeline(
//         &mut self,
//         state: &'a ComputeState,
//     ) {
//         let cmd = ComputeCmd::BindPipeline { state };
//         self.cmds.push(cmd);
//     }
//     pub fn bind_descriptor<T>(
//         &mut self,
//         descriptor: &T,
//     ) where
//         T: DescriptorInfo,
//     {
//         let mut d = self.pool_allocator.allocate::<T>();
//         d.update(&self.fg.ctx, descriptor, &self.fg);
//         let cmd = ComputeCmd::BindDescriptor(d.handle);
//         self.cmds.push(cmd);
//     }
// }
// pub enum GraphicsCmd<'a> {
//     BindVertex(BufferHandle),
//     BindIndex(BufferHandle),
//     BindDescriptor(DescriptorHandle),
//     BindPipeline {
//         state: &'a GraphicsPipelineState,
//         stride: u32,
//         vertex_input_data: Vec<VertexInputData>,
//     },
//     DrawIndex {
//         len: u32,
//     },
// }

// pub struct GraphicsCommandbuffer<'a> {
//     fg: &'a Framegraph<Compiled>,
//     pool_allocator: Allocator<'a>,
//     pub(crate) cmds: Vec<GraphicsCmd<'a>>,
// }

// impl<'a> GraphicsCommandbuffer<'a> {
//     pub fn new(
//         fg: &'a Framegraph<Compiled>,
//         pool_allocator: Allocator<'a>,
//     ) -> Self {
//         GraphicsCommandbuffer {
//             fg,
//             cmds: Vec::new(),
//             pool_allocator,
//         }
//     }
//     pub fn bind_vertex<T>(
//         &mut self,
//         buffer: Buffer<T>,
//     ) {
//         let cmd = GraphicsCmd::BindVertex(buffer.buffer);
//         self.cmds.push(cmd);
//     }
//     pub fn bind_index(
//         &mut self,
//         buffer: Buffer<u32>,
//     ) {
//         let cmd = GraphicsCmd::BindIndex(buffer.buffer);
//         self.cmds.push(cmd);
//     }
//     pub fn bind_pipeline<T: VertexInput>(
//         &mut self,
//         state: &'a GraphicsPipelineState,
//     ) {
//         let cmd = GraphicsCmd::BindPipeline {
//             state,
//             stride: std::mem::size_of::<T>() as u32,
//             vertex_input_data: T::vertex_input_data(),
//         };
//         self.cmds.push(cmd);
//     }
//     pub fn draw_index(
//         &mut self,
//         len: usize,
//     ) {
//         let cmd = GraphicsCmd::DrawIndex { len: len as u32 };
//         self.cmds.push(cmd);
//     }

//     pub fn bind_descriptor<T>(
//         &mut self,
//         descriptor: &T,
//     ) where
//         T: DescriptorInfo,
//     {
//         let mut d = self.pool_allocator.allocate::<T>();
//         d.update(&self.fg.ctx, descriptor, &self.fg);
//         let cmd = GraphicsCmd::BindDescriptor(d.handle);
//         self.cmds.push(cmd);
//     }
// }
