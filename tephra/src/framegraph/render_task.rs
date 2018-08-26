use super::Blackboard;
use context::Context;
use framegraph::{Compiled, Framegraph};
use render::Render;
use std::sync::Arc;
use std::ops::Deref;

pub type ARenderTask<T> = Arc<RenderTask<T>>;
pub struct RenderTask<T> {
    pub data: T,
    pub execute: fn(&T, &Blackboard, &Render, &Framegraph<Compiled>),
}

impl<T> Deref for RenderTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


pub trait Execute {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Framegraph<Compiled>);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, blackboard: &Blackboard, render: &Render, ctx: &Framegraph<Compiled>) {
        (self.execute)(&self.data, blackboard, render, ctx)
    }
}
