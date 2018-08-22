use context::Context;
use render::Render;
pub struct RenderTask<T> {
    pub data: T,
    pub execute: fn(&T, &Render, &Context),
}

pub trait Execute {
    fn execute(&self, render: &Render, ctx: &Context);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, render: &Render, ctx: &Context) {
        (self.execute)(&self.data, render, ctx)
    }
}
