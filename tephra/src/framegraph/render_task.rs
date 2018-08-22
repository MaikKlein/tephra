use super::Blackboard;
use context::Context;
use render::Render;

pub struct RenderTask<T> {
    pub data: T,
    pub execute: fn(&T, &Blackboard, &Render, &Context),
}

pub trait Execute {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Context);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Context) {
        (self.execute)(&self.data, blackboard, render, ctx)
    }
}
