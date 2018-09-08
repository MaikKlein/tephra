use commandbuffer::GraphicsCommandbuffer;
use descriptor::DescriptorInfo;
use framegraph::blackboard::Blackboard;
use framegraph::{Compiled, Framegraph, Resource, TaskBuilder};
use image::Image;
use render::Render;
use renderpass::VertexInput;
use std::ops::Deref;
use std::sync::Arc;

pub trait Renderpass<'graph> {
    type Input;
    type Vertex: VertexInput;
    type Descriptor: DescriptorInfo;
    fn setup<'a>(&'a self, task_builder: &mut TaskBuilder<'a, 'graph>) -> Self::Input;
    fn framebuffer(&self, data: &Self::Input) -> Vec<Resource<Image>>;
    fn execute<'a>(
        data: &Self::Input,
        cmds: &mut GraphicsCommandbuffer<'a>,
        fg: &Framegraph<'graph, Compiled>,
    );
}
pub type ExecuteFn<'graph, T> =
    for<'a> fn(&T, &'a Blackboard, &mut GraphicsCommandbuffer<'a>, &Framegraph<'graph, Compiled>);
pub type ARenderTask<'graph, T> = Arc<RenderTask<'graph, T>>;
pub struct RenderTask<'graph, T> {
    pub data: T,
    pub execute: ExecuteFn<'graph, T>,
}

impl<'graph, T> Deref for RenderTask<'graph, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait Execute<'graph> {
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        render: &mut GraphicsCommandbuffer<'a>,
        ctx: &Framegraph<'graph, Compiled>,
    );
}

impl<'graph, T> Execute<'graph> for RenderTask<'graph, T> {
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        render: &mut GraphicsCommandbuffer<'a>,
        ctx: &Framegraph<'graph, Compiled>,
    ) {
        (self.execute)(&self.data, blackboard, render, ctx)
    }
}
