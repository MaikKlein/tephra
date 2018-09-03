use commandbuffer::GraphicsCommandbuffer;
use framegraph::blackboard::Blackboard;
use framegraph::{Compiled, Framegraph, Resource, TaskBuilder};
use image::Image;
use render::Render;
use std::ops::Deref;
use std::sync::Arc;

pub trait Renderpass {
    type Input;
    fn setup(task_builder: &mut TaskBuilder) -> Self::Input;
    fn framebuffer(data: &Self::Input) -> Vec<Resource<Image>>;
    fn execute(
        data: &Self::Input,
        cmds: &mut GraphicsCommandbuffer,
        fg: &Framegraph<Compiled>,
    );
}
pub type ExecuteFn<T> =
    for<'a> fn(&T, &'a Blackboard, &mut GraphicsCommandbuffer<'a>, &Framegraph<Compiled>);
pub type ARenderTask<T> = Arc<RenderTask<T>>;
pub struct RenderTask<T> {
    pub data: T,
    pub execute: ExecuteFn<T>,
}

impl<T> Deref for RenderTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait Execute {
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        render: &mut GraphicsCommandbuffer<'a>,
        ctx: &Framegraph<Compiled>,
    );
}

impl<T> Execute for RenderTask<T> {
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        render: &mut GraphicsCommandbuffer<'a>,
        ctx: &Framegraph<Compiled>,
    ) {
        (self.execute)(&self.data, blackboard, render, ctx)
    }
}
