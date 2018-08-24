use super::Blackboard;
use context::Context;
use framegraph::{Compiled, Framegraph};
use render::Render;

pub struct RenderTask<T> {
    pub data: T,
    pub execute: fn(&T, &Blackboard, &Render, &Framegraph<Compiled>),
}

pub trait Execute {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Framegraph<Compiled>);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Framegraph<Compiled>) {
        (self.execute)(&self.data, blackboard, render, ctx)
    }
}
