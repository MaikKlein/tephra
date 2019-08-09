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
pub struct ComputePipelineState {
    pub compute_shader: ShaderStage,
    pub layout: Vec<Binding<DescriptorType>>,
}
#[derive(Default)]
pub struct ComputePipelineStateBuilder {
    pub compute_shader: Option<ShaderStage>,
    pub layout: Option<Vec<Binding<DescriptorType>>>,
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
    pub fn build(self) -> Option<ComputePipelineState> {
        let compute_shader = self.compute_shader?;
        let layout = self.layout?;
        Some(ComputePipelineState {
            compute_shader,
            layout,
        })
    }
    pub unsafe fn create(self, ctx: &Context) -> ComputePipeline {
        ctx.create_compute_pipeline(&self.build().unwrap())
    }
    pub fn compute_shader(mut self, shader: ShaderStage) -> Self {
        self.compute_shader = Some(shader);
        self
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
pub struct GraphicsPipelineState {
    pub vertex_shader: ShaderStage,
    pub fragment_shader: ShaderStage,
    pub render_target: Renderpass,
    pub layout: Vec<Binding<DescriptorType>>,
    // TODO: Default to SoA not AoS
    pub vertex_input: (Stride, Vec<VertexInputData>),
}
#[derive(Default)]
pub struct GraphicsPipelineStateBuilder {
    pub vertex_shader: Option<ShaderStage>,
    pub fragment_shader: Option<ShaderStage>,
    pub render_target: Option<Renderpass>,
    pub layout: Option<Vec<Binding<DescriptorType>>>,
    pub vertex_input: Option<(Stride, Vec<VertexInputData>)>,
}
impl GraphicsPipelineStateBuilder {
    pub fn build(self) -> Option<GraphicsPipelineState> {
        let vertex_shader = self.vertex_shader?;
        let fragment_shader = self.fragment_shader?;
        let render_target = self.render_target?;
        let layout = self.layout?;
        let vertex_input = self.vertex_input?;
        Some(GraphicsPipelineState {
            vertex_shader,
            fragment_shader,
            render_target,
            layout,
            vertex_input,
        })
    }
    pub fn render_target(mut self, target: Renderpass) -> Self {
        self.render_target = Some(target);
        self
    }
    pub fn vertex_shader(mut self, shader: ShaderStage) -> Self {
        self.vertex_shader = Some(shader);
        self
    }
    pub fn fragment_shader(mut self, shader: ShaderStage) -> Self {
        self.fragment_shader = Some(shader);
        self
    }
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
