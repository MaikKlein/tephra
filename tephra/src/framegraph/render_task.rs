use commandbuffer::{ComputeCommandbuffer, GraphicsCommandbuffer};
use descriptor::DescriptorInfo;
use framegraph::blackboard::Blackboard;
use framegraph::{Compiled, Framegraph, Resource, TaskBuilder};
use image::Image;
use render::Render;
use renderpass::VertexInput;
use std::ops::Deref;
use std::sync::Arc;

pub trait Renderpass {
    type Vertex: VertexInput;
    type Layout: DescriptorInfo;
    fn framebuffer(&self) -> Vec<Resource<Image>>;
    fn execute<'cmd>(
        &'cmd self,
        &'cmd Blackboard,
        cmds: &mut GraphicsCommandbuffer<'cmd>,
        fg: &Framegraph<Compiled>,
    );
}
// pub type ExecuteFn<, T> =
//     for<'a> fn(&T, &'a Blackboard, &mut GraphicsCommandbuffer<'a>, &Framegraph<, Compiled>);
// pub type ARenderTask<, T> = Arc<RenderTask<, T>>;
// pub struct RenderTask<, T> {
//     pub data: T,
//     pub execute: ExecuteFn<, T>,
// }

// impl<, T> Deref for RenderTask<, T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         &self.data
//     }
// }

pub trait ExecuteGraphics {
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut GraphicsCommandbuffer<'cmd>,
        ctx: &Framegraph<Compiled>,
    );
}

pub trait Computepass {
    type Layout: DescriptorInfo;
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        cmds: &mut ComputeCommandbuffer<'cmd>,
        fg: &Framegraph<Compiled>,
    );
}

pub trait ExecuteCompute {
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut ComputeCommandbuffer<'cmd>,
        ctx: &Framegraph<Compiled>,
    );
}

impl<P> ExecuteCompute for P
where
    P: Computepass,
{
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut ComputeCommandbuffer<'cmd>,
        ctx: &Framegraph<Compiled>,
    ) {
        self.execute(blackboard, render, ctx)
    }
}
impl<P> ExecuteGraphics for P
where
    P: Renderpass,
{
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        render: &mut GraphicsCommandbuffer<'cmd>,
        ctx: &Framegraph<Compiled>,
    ) {
        self.execute(blackboard, render, ctx)
    }
}
