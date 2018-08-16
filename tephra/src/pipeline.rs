use backend::BackendApi;
use context::Context;
use renderpass::{Pass, Renderpass};
use shader::Shader;
use std::marker::PhantomData;
use downcast::Downcast;
pub trait VertexInput {}

pub trait CreatePipeline {
    fn from_pipeline_builder(&self, pipline_builder: PipelineBuilder) -> Pipeline;
}

pub trait PipelineApi: Downcast {
}
impl_downcast!(PipelineApi);

pub struct PipelineBuilder<'a> {
    pub vertex_shader: Option<&'a Shader>,
    pub fragment_shader: Option<&'a Shader>,
    pub renderpass: Option<&'a Renderpass>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new() -> Self {
        PipelineBuilder {
            vertex_shader: None,
            fragment_shader: None,
            renderpass: None,
        }
    }

    pub fn with_vertex_shader(self, shader: &'a Shader) -> Self {
        PipelineBuilder {
            vertex_shader: Some(shader),
            ..self
        }
    }

    pub fn with_renderpass(self, renderpass: &'a Renderpass) -> Self {
        PipelineBuilder {
            renderpass: Some(renderpass),
            ..self
        }
    }
    pub fn with_fragment_shader(self, shader: &'a Shader) -> Self {
        PipelineBuilder {
            fragment_shader: Some(shader),
            ..self
        }
    }

    pub fn build(self, ctx: &Context) -> Pipeline {
        ctx.from_pipeline_builder(self)
    }
}

pub struct Pipeline {
    pub data: Box<dyn PipelineApi>,
}
impl Pipeline {
    pub fn downcast<B: BackendApi>(&self) -> &B::Pipeline {
        self.data.downcast_ref::<B::Pipeline>().expect("Vulkan Backend Pipeline")
    }
}
