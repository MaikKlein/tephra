//use renderpass::{Pass, Renderpass};
use shader::Shader;

// pub trait CreatePipeline {
//     fn from_pipeline_builder(&self, pipline_builder: PipelineState) -> Pipeline;
// }

// pub trait PipelineApi: Downcast {
// }
// impl_downcast!(PipelineApi);

pub struct PipelineState {
    pub vertex_shader: Option<Shader>,
    pub fragment_shader: Option<Shader>,
}

impl PipelineState {
    pub fn new() -> Self {
        PipelineState {
            vertex_shader: None,
            fragment_shader: None,
        }
    }

    pub fn with_vertex_shader(self, shader: Shader) -> Self {
        PipelineState {
            vertex_shader: Some(shader),
            ..self
        }
    }

    pub fn with_fragment_shader(self, shader: Shader) -> Self {
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
