use crate::backend::BackendApi;
use crate::buffer::{Buffer, BufferApi};
use crate::commandbuffer::{ComputeCmd, GraphicsCmd};
use crate::context::Context;
use crate::descriptor::NativeLayout;
use crate::downcast::Downcast;
use crate::framegraph::{Compiled, Framegraph};
use crate::image::{Image, Resolution};
use crate::renderpass::{VertexInput, VertexInputData};
use std::mem::size_of;
use std::ops::Deref;

pub trait CreateRender {
    fn create_render(
        &self,
        resolution: Resolution,
        images: &[Image],
        layout: &NativeLayout,
    ) -> Render;
}

pub trait CreateCompute {
    fn create_compute(&self, layout: &NativeLayout) -> Compute;
}

pub trait ComputeApi: Downcast {
    fn execute_commands(&self, cmds: &[ComputeCmd]);
}
impl_downcast!(ComputeApi);

pub struct Compute {
    pub inner: Box<dyn ComputeApi>,
}
impl Deref for Compute {
    type Target = ComputeApi;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
impl Compute {
    pub fn new(ctx: &Context, layout: &NativeLayout) -> Compute {
        ctx.create_compute(layout)
    }
}

pub trait RenderApi: Downcast {
    fn execute_commands(&self, cmds: &[GraphicsCmd]);
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
        images: &[Image],
        layout: &NativeLayout,
    ) -> Render {
        ctx.create_render(resolution, images, layout)
    }
}
