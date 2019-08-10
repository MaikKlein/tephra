use crate::{
    buffer::{Buffer, BufferHandle},
    descriptor::{DescriptorHandle, DescriptorInfo, DescriptorType, Pool},
    image::ImageHandle,
    pipeline::{ComputePipeline, GraphicsPipeline, GraphicsPipelineState},
    renderpass::{Framebuffer, Renderpass, VertexInput, VertexInputData},
};
use bitflags::bitflags;
use smallvec::SmallVec;
use std::{
    hash::Hasher,
    marker::PhantomData,
    ops::{Deref, Range},
};

bitflags! {
    pub struct AccessFlags: u32 {
        const TRANSFER_READ = 1 << 0;
        const TRANSFER_WRITE = 1 << 1;
        const VERTEX_BUFFER = 1 << 2;
        const INDEX_BUFFER = 1 << 3;
        const COMPUTE_READ = 1 << 4;
        const COMPUTE_WRITE = 1 << 5;
        const FRAGMENT_READ = 1 << 6;
        const FRAGMENT_READ_COLOR = 1 << 7;
    }
}
#[derive(Default)]
pub struct ShaderArguments(SmallVec<[(u32, Descriptor); MAX_SHADER_ARGS]>);
impl ShaderArguments {
    pub fn builder() -> ShaderArumentsBuilder {
        ShaderArumentsBuilder {
            space: ShaderArguments::default(),
        }
    }
}

pub struct ShaderArumentsBuilder {
    space: ShaderArguments,
}
impl Deref for ShaderArguments {
    type Target = [(u32, Descriptor)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ShaderArumentsBuilder {
    pub fn with_shader_arg(mut self, set: u32, shader_args: Descriptor) -> Self {
        self.space.0.push((set, shader_args));
        self
    }
    pub fn build(self) -> ShaderArguments {
        self.space
    }
}
#[derive(Copy, Clone, Debug)]
pub enum ShaderResource {
    Buffer(BufferHandle),
    Image(ImageHandle),
}
impl<T> From<Buffer<T>> for ShaderResource {
    fn from(buffer: Buffer<T>) -> ShaderResource {
        ShaderResource::Buffer(buffer.buffer)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Access {
    Read,
    Write,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ShaderView {
    pub binding: u32,
    pub ty: DescriptorType,
    pub access: Access,
}
const MAX_SHADER_ARGS: usize = 4;

pub type StackVec<T> = SmallVec<[T; MAX_SHADER_ARGS]>;
pub type ShaderViews = StackVec<ShaderView>;
pub type ShaderResources = StackVec<ShaderResource>;

#[derive(Default, Debug)]
pub struct Descriptor {
    pub resources: ShaderResources,
    pub views: ShaderViews,
}

pub struct DescriptorBuilder {
    descriptor: Descriptor,
}
impl DescriptorBuilder {
    pub fn build(self) -> Descriptor {
        self.descriptor
    }
    pub fn with<S: Into<ShaderResource>>(
        mut self,
        shader_resource: S,
        binding: u32,
        ty: DescriptorType,
        access: Access,
    ) -> Self {
        let view = ShaderView {
            binding,
            ty,
            access,
        };
        self.descriptor.views.push(view);
        self.descriptor.resources.push(shader_resource.into());
        self
    }
}
impl Descriptor {
    pub fn builder() -> DescriptorBuilder {
        DescriptorBuilder {
            descriptor: Descriptor::default(),
        }
    }
}

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
    pub shader_arguments: ShaderArguments,
    pub range: Range<u32>,
}

pub struct DispatchCommand {
    pub pipeline: ComputePipeline,
    pub shader_arguments: ShaderArguments,
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
    pub fn record<'a, Q>(&'a mut self) -> RecordCommandList<'a, Q> {
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
    pub fn draw_indexed<Vertex>(
        mut self,
        graphics_pipeline: GraphicsPipeline,
        renderpass: Renderpass,
        framebuffer: Framebuffer,
        shader_arguments: ShaderArguments,
        vertex_buffer: Buffer<Vertex>,
        index_buffer: Buffer<u32>,
        range: Range<u32>,
    ) -> Self
    where
        Vertex: VertexInput,
    {
        let cmd = DrawCommand {
            graphics_pipeline,
            renderpass,
            framebuffer,
            shader_arguments,
            vertex: vertex_buffer.buffer,
            index: index_buffer,
            range,
        };
        self.commands.push(Command::Draw(cmd));
        self
    }
}
impl RecordCommandList<'_, Compute> {
    pub fn dispatch(
        mut self,
        pipeline: ComputePipeline,
        shader_arguments: ShaderArguments,
        x: u32,
        y: u32,
        z: u32,
    ) -> Self {
        let cmd = DispatchCommand {
            pipeline,
            shader_arguments,
            x,
            y,
            z,
        };
        self.commands.push(Command::Dispatch(cmd));
        self
    }
}

pub enum Command {
    CopyImage(CopyImage),
    Draw(DrawCommand),
    Dispatch(DispatchCommand),
}
pub trait SubmitApi {
    unsafe fn submit_commands(&self, pool: &mut Pool, commands: &CommandList);
}
