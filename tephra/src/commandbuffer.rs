use buffer::{GenericBuffer, Buffer, BufferApi};
use framegraph::{ResourceIndex,  Resource };
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

pub enum GraphicsCmd {
    BindVertex(ResourceIndex),
    BindIndex(ResourceIndex),
    BindPipeline {
        state: PipelineState,
        stride: u32,
        vertex_input_data: Vec<VertexInputData>,
    },
    DrawIndex {
        len: u32,
    },
}

pub struct GraphicsCommandbuffer {
    pub(crate) cmds: Vec<GraphicsCmd>,
}

impl GraphicsCommandbuffer {
    pub fn new() -> Self {
        GraphicsCommandbuffer { cmds: Vec::new() }
    }
    pub fn bind_vertex(&mut self, buffer: Resource<GenericBuffer>) {
        let cmd = GraphicsCmd::BindVertex(buffer.id);
        self.cmds.push(cmd);
    }
    pub fn bind_index(&mut self, buffer: Resource<GenericBuffer>) {
        let cmd = GraphicsCmd::BindIndex(buffer.id);
        self.cmds.push(cmd);
    }
    pub fn bind_pipeline<T: VertexInput>(&mut self, state: PipelineState) {
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
}
