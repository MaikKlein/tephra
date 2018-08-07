use backend::BackendApi;
use context::Context;
use std::marker::PhantomData;
pub trait VertexInput {}

pub trait PipelineApi<Backend>
where
    Backend: BackendApi,
{
    fn from_pipeline_builder(
        context: &Context<Backend>,
        pipline_builder: &PipelineBuilder<Backend>,
    ) -> Self;
}

#[derive(Clone)]
pub struct PipelineBuilder<Backend>
where
    Backend: BackendApi,
{
    context: Context<Backend>,
}

impl<Backend> PipelineBuilder<Backend>
where
    Backend: BackendApi,
{
    pub fn new(context: &Context<Backend>) -> Self {
        PipelineBuilder {
            context: context.clone(),
        }
    }
    pub fn build(self) {}
}

pub struct Pipeline<Backend>
where
    Backend: BackendApi,
{
    data: Backend::Pipeline,
}
