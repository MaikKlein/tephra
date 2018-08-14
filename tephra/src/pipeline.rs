use backend::BackendApi;
use context::Context;
use renderpass::{Pass, Renderpass};
use shader::Shader;
use std::marker::PhantomData;
pub trait VertexInput {}

pub trait PipelineApi<Backend>
where
    Backend: BackendApi,
{
    fn from_pipeline_builder<P: Pass>(
        context: &Context<Backend>,
        pipline_builder: PipelineBuilder<P, Backend>,
    ) -> Self;
}

pub struct PipelineBuilder<'a, P: Pass + 'a, Backend>
where
    Backend: BackendApi,
{
    pub vertex_shader: Option<Shader<Backend>>,
    pub fragment_shader: Option<Shader<Backend>>,
    pub renderpass: Option<&'a Renderpass<P, Backend>>,
    _m: PhantomData<Backend>,
}

impl<'a, P, Backend> PipelineBuilder<'a, P, Backend>
where
    P: Pass + 'a,
    Backend: BackendApi,
    Backend::Pipeline: PipelineApi<Backend>,
{
    pub fn new() -> Self {
        PipelineBuilder {
            _m: PhantomData,
            vertex_shader: None,
            fragment_shader: None,
            renderpass: None,
        }
    }

    pub fn with_vertex_shader(self, shader: Shader<Backend>) -> Self {
        PipelineBuilder {
            vertex_shader: Some(shader),
            ..self
        }
    }

    pub fn with_renderpass(self, renderpass: &'a Renderpass<P, Backend>) -> Self {
        PipelineBuilder {
            renderpass: Some(renderpass),
            ..self
        }
    }
    pub fn with_fragment_shader(self, shader: Shader<Backend>) -> Self {
        PipelineBuilder {
            fragment_shader: Some(shader),
            ..self
        }
    }

    pub fn build(self, context: &Context<Backend>) -> Pipeline<Backend> {
        let data = PipelineApi::from_pipeline_builder(context, self);
        Pipeline { data }
    }
}

pub struct Pipeline<Backend>
where
    Backend: BackendApi,
{
    pub data: Backend::Pipeline,
}
