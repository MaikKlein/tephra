//use renderpass::{Pass, Renderpass};
use crate::{
    descriptor::{Binding, DescriptorInfo, DescriptorResource, DescriptorType},
    renderpass::{self, RenderTarget, VertexInput, VertexInputData},
    shader::ShaderModule,
};
use derive_builder::Builder;
use slotmap::new_key_type;

new_key_type!(
    pub struct GraphicsPipeline;
    pub struct ComputePipeline;
);

// pub trait CreatePipeline {
//     fn from_pipeline_builder(&self, pipline_builder: PipelineState) -> Pipeline;
// }

// pub trait PipelineApi: Downcast {
// }
// impl_downcast!(PipelineApi);

pub trait PipelineApi {
    unsafe fn create_graphics_pipeline(&self, state: &PipelineState) -> GraphicsPipeline;
}
#[derive(Clone)]
pub struct ComputeState {
    pub compute_shader: Option<ShaderModule>,
}

#[derive(Clone)]
pub struct ShaderStage {
    pub shader_module: ShaderModule,
    pub entry_name: String,
}
pub type Stride = u32;
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct PipelineState {
    pub vertex_shader: ShaderStage,
    pub fragment_shader: ShaderStage,
    pub render_target: RenderTarget,
    #[builder(setter(skip = "false"))]
    pub layout: Vec<Binding<DescriptorType>>,
    #[builder(setter(skip = "false"))]
    // TODO: Default to SoA not AoS
    pub vertex_input: (Stride, Vec<VertexInputData>),
}
impl PipelineStateBuilder {
    pub fn layout<D: DescriptorInfo>(mut self) -> Self {
        self.layout = Some(D::layout());
        self
    }
    pub fn vertex<V: VertexInput>(mut self) -> Self {
        self.vertex_input = Some((std::mem::size_of::<V>() as Stride, V::vertex_input_data()));
        self
    }
}

// pub struct Pipeline {
//     pub data: Box<dyn PipelineApi>,
// }
// impl Pipeline {
//     pub fn downcast<B: BackendApi>(&self) -> &B::Pipeline {
//         self.data.downcast_ref::<B::Pipeline>().expect("Vulkan Backend Pipeline")
//     }
// }
