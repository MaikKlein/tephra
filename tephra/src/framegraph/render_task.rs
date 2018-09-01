use commandbuffer::GraphicsCommandbuffer;
use framegraph::blackboard::Blackboard;
use framegraph::{Compiled, Framegraph, Resource, TaskBuilder};
use image::Image;
use render::Render;
use std::ops::Deref;
use std::sync::Arc;

// pub trait Renderpass {
//     type Framebuffer;
//     fn setup(task_builder: &mut TaskBuilder) -> Self::Framebuffer;
//     fn framebuffer(data: &Self::Framebuffer) -> Vec<Resource<Image>>;
//     fn execute(
//         data: &Self::Framebuffer,
//         cmds: &mut GraphicsCommandbuffer,
//         fg: &Framegraph<Compiled>,
//     );
// }
pub type ARenderTask<T> = Arc<RenderTask<T>>;
pub struct RenderTask<T> {
    pub data: T,
    pub execute: fn(&T, &mut GraphicsCommandbuffer, &Framegraph<Compiled>),
}

impl<T> Deref for RenderTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait Execute {
    fn execute(&self, render: &mut GraphicsCommandbuffer, ctx: &Framegraph<Compiled>);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, render: &mut GraphicsCommandbuffer, ctx: &Framegraph<Compiled>) {
        (self.execute)(&self.data, render, ctx)
    }
}
