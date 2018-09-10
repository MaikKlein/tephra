use commandbuffer::{ComputeCommandbuffer, GraphicsCommandbuffer};
use descriptor::DescriptorInfo;
use framegraph::blackboard::Blackboard;
use framegraph::{Compiled, Framegraph, Resource, TaskBuilder};
use image::Image;
use render::Render;
use renderpass::VertexInput;
use std::ops::Deref;
use std::sync::Arc;

pub trait Renderpass<'graph> {
    type Vertex: VertexInput;
    type Layout: DescriptorInfo;
    fn framebuffer(&self) -> Vec<Resource<Image>>;
    fn execute<'a>(
        &self,
        &'a Blackboard,
        cmds: &mut GraphicsCommandbuffer<'a>,
        fg: &Framegraph<'graph, Compiled>,
    );
}
// pub type ExecuteFn<'graph, T> =
//     for<'a> fn(&T, &'a Blackboard, &mut GraphicsCommandbuffer<'a>, &Framegraph<'graph, Compiled>);
// pub type ARenderTask<'graph, T> = Arc<RenderTask<'graph, T>>;
// pub struct RenderTask<'graph, T> {
//     pub data: T,
//     pub execute: ExecuteFn<'graph, T>,
// }

// impl<'graph, T> Deref for RenderTask<'graph, T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         &self.data
//     }
// }

pub trait ExecuteGraphics<'graph> {
    fn execute<'cmd>(
        &self,
        blackboard: &'cmd Blackboard,
        render: &mut GraphicsCommandbuffer<'cmd>,
        ctx: &Framegraph<'graph, Compiled>,
    );
}

pub trait Computepass<'graph> {
    type Layout: DescriptorInfo;
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        cmds: &mut ComputeCommandbuffer<'cmd>,
        fg: &Framegraph<'graph, Compiled>,
    );
}

pub trait ExecuteCompute<'graph> {
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut ComputeCommandbuffer<'cmd>,
        ctx: &Framegraph<'graph, Compiled>,
    );
}

impl<'graph, P> ExecuteCompute<'graph> for P
where
    P: Computepass<'graph>,
{
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut ComputeCommandbuffer<'cmd>,
        ctx: &Framegraph<'graph, Compiled>,
    ) {
        self.execute(blackboard, render, ctx)
    }
}
impl<'graph, P> ExecuteGraphics<'graph> for P
where
    P: Renderpass<'graph>,
{
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        render: &mut GraphicsCommandbuffer<'a>,
        ctx: &Framegraph<'graph, Compiled>,
    ) {
        self.execute(blackboard, render, ctx)
    }
}
