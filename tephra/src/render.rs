use backend::BackendApi;
use buffer::{Buffer, BufferApi};
use commandbuffer::{ComputeCmd, GraphicsCmd };
use context::Context;
use downcast::Downcast;
use framegraph::{Compiled, Framegraph};
use image::{Image, Resolution};
use pipeline::PipelineState;
use renderpass::{VertexInput, VertexInputData};
use descriptor::NativeLayout;
use std::mem::size_of;
use std::ops::Deref;

pub trait CreateRender {
    fn create_render(
        &self,
        resolution: Resolution,
        images: &[&Image],
        layout: &NativeLayout,
    ) -> Render;
}

pub trait CreateCompute {
    fn create_compute(
        &self,
        layout: &NativeLayout,
    ) -> Compute;
}

pub trait ComputeApi: Downcast {
    fn execute_commands(
        &self,
        cmds: &[ComputeCmd],
    );
}
impl_downcast!(ComputeApi);

pub struct Compute {
    pub inner: Box<dyn ComputeApi>
}

pub trait RenderApi: Downcast {
    fn execute_commands(
        &self,
        cmds: &[GraphicsCmd],
    );
}
impl_downcast!(RenderApi);

pub struct Render {
    pub inner: Box<dyn RenderApi>,
}
impl Deref for Render {
    type Target = RenderApi;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl RenderApi {
    pub fn downcast<B: BackendApi>(&self) -> &B::Render {
        self.downcast_ref::<B::Render>()
            .expect("Downcast Render Vulkan")
    }
}
impl Render {
    pub fn new(
        ctx: &Context,
        resolution: Resolution,
        images: &[&Image],
        layout: &NativeLayout,
    ) -> Render {
        ctx.create_render(resolution, images, layout)
    }
}
