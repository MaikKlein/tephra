use crate::{
    context::Context,
    descriptor::{Binding, DescriptorInfo, DescriptorType},
    renderpass::{Renderpass, VertexInput, VertexInputData},
    shader::ShaderModule,
};
use derive_builder::Builder;
use slotmap::new_key_type;

new_key_type!(
    pub struct GraphicsPipeline;
    pub struct ComputePipeline;
);

pub trait PipelineApi {
    unsafe fn create_graphics_pipeline(&self, state: &GraphicsPipelineState) -> GraphicsPipeline;
    unsafe fn create_compute_pipeline(&self, state: &ComputePipelineState) -> ComputePipeline;
}
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct ComputePipelineState {
    pub compute_shader: ShaderStage,
    #[builder(setter(skip = "false"), private)]
    pub layout: Vec<Binding<DescriptorType>>,
}
impl ComputePipeline {
    pub fn builder() -> ComputePipelineStateBuilder {
        Default::default()
    }
}
impl GraphicsPipeline {
    pub fn builder() -> GraphicsPipelineStateBuilder {
        Default::default()
    }
}
impl ComputePipelineStateBuilder {
    pub unsafe fn create(self, ctx: &Context) -> ComputePipeline {
        ctx.create_compute_pipeline(&self.build().unwrap())
    }
    pub fn layout<D: DescriptorInfo>(mut self) -> Self {
        self.layout = Some(D::layout());
        self
    }
}

#[derive(Clone)]
pub struct ShaderStage {
    pub shader_module: ShaderModule,
    pub entry_name: String,
}
pub type Stride = u32;
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct GraphicsPipelineState {
    pub vertex_shader: ShaderStage,
    pub fragment_shader: ShaderStage,
    pub render_target: Renderpass,
    #[builder(setter(skip = "false"), private)]
    pub layout: Vec<Binding<DescriptorType>>,
    #[builder(setter(skip = "false"))]
    // TODO: Default to SoA not AoS
    pub vertex_input: (Stride, Vec<VertexInputData>),
}
impl GraphicsPipelineStateBuilder {
    pub unsafe fn create(self, ctx: &Context) -> GraphicsPipeline {
        let state = self.build().unwrap();
        ctx.create_graphics_pipeline(&state)
    }
    pub fn layout<D: DescriptorInfo>(mut self) -> Self {
        self.layout = Some(D::layout());
        self
    }
    pub fn vertex<V: VertexInput>(mut self) -> Self {
        self.vertex_input = Some((std::mem::size_of::<V>() as Stride, V::vertex_input_data()));
        self
    }
}
