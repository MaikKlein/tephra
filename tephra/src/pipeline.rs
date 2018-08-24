use backend::BackendApi;
use context::Context;
//use renderpass::{Pass, Renderpass};
use shader::Shader;
use std::marker::PhantomData;
use downcast::Downcast;

// pub trait CreatePipeline {
//     fn from_pipeline_builder(&self, pipline_builder: PipelineState) -> Pipeline;
// }

// pub trait PipelineApi: Downcast {
// }
// impl_downcast!(PipelineApi);

pub struct PipelineState<'a> {
    pub vertex_shader: Option<&'a Shader>,
    pub fragment_shader: Option<&'a Shader>,
}

impl<'a> PipelineState<'a> {
    pub fn new() -> Self {
        PipelineState {
            vertex_shader: None,
            fragment_shader: None,
        }
    }

    pub fn with_vertex_shader(self, shader: &'a Shader) -> Self {
        PipelineState {
            vertex_shader: Some(shader),
            ..self
        }
    }

    pub fn with_fragment_shader(self, shader: &'a Shader) -> Self {
        PipelineState {
            fragment_shader: Some(shader),
            ..self
        }
    }

    // pub fn build(self, ctx: &Context) -> Pipeline {
    //     ctx.from_pipeline_builder(self)
    // }
}

// pub struct Pipeline {
//     pub data: Box<dyn PipelineApi>,
// }
// impl Pipeline {
//     pub fn downcast<B: BackendApi>(&self) -> &B::Pipeline {
//         self.data.downcast_ref::<B::Pipeline>().expect("Vulkan Backend Pipeline")
//     }
// }
