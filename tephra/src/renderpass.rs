use ash::vk;
use backend::BackendApi;
use buffer::{Buffer, BufferProperty};
use context::Context;
use downcast::Downcast;
use image::RenderTarget;
use image::RenderTargetInfo;
use pipeline::PipelineState;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub enum VertexType {
    F32(usize),
}
impl VertexType {
    pub fn size(self) -> usize {
        match self {
            VertexType::F32(n) => std::mem::size_of::<f32>() * n,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VertexInputData {
    pub vertex_type: VertexType,
    pub binding: u32,
    pub location: u32,
    pub offset: u32,
}
pub trait VertexTypeData {
    fn vertex_type() -> VertexType;
}

impl VertexTypeData for [f32; 4] {
    fn vertex_type() -> VertexType {
        VertexType::F32(4)
    }
}

pub trait VertexInput {
    fn vertex_input_data() -> Vec<VertexInputData>;
}
// pub trait RenderApi {
//     fn new(context: &Context<Backend>) -> Self;
//     // fn draw_indexed<P, Vertex, Index>(
//     //     &self,
//     //     frame_buffer: vk::Framebuffer,
//     //     renderpass: &Renderpass<P, Backend>,
//     //     pipeline: Pipeline<Backend>,
//     //     vertex: &Buffer<Vertex, impl BufferProperty, Backend>,
//     //     index: &Buffer<Index, impl BufferProperty, Backend>,
//     // ) where
//     //     P: Pass;
// }

// pub struct Render<Backend: BackendApi> {
//     pub data: Backend::Render,
// }

//pub trait Pass<'a> {
//    type Input: VertexInput;
//    type Target: RenderTarget<'a>;
//    //fn render<Backend: BackendApi>(&self, render: &Render<Backend>) {}
//}

//pub trait CreateRenderpass {
//    fn new(&self, vertex_input: &[VertexInputData]) -> Renderpass;
//}

//pub trait RenderpassApi: Downcast {}
//impl_downcast!(RenderpassApi);

// pub struct Renderpass {
//     pub renderpass: Box<dyn RenderpassApi>,
// }
// impl Renderpass {
//     pub fn downcast<B: BackendApi>(&self) -> &B::Renderpass {
//         self.renderpass.downcast_ref::<B::Renderpass>().unwrap()
//     }
//     pub fn new<'a, P: Pass<'a>>(context: &Context, _p: P) -> Self {
//         CreateRenderpass::new(context.context.as_ref(), &P::Input::vertex_input_data())
//     }
// }

// impl<P> Renderpass<P>
// where
//     P: Pass,
// {
//     pub fn new(ctx: &Context, pass: P) -> Self {
//         CreateRenderpass::new(ctx, pass)
//     }
//     pub fn render(&self) {
//         // let render = Render {
//         // }
//     }
// }

// impl<P, Backend> Deref for Renderpass<P, Backend>
// where
//     P: Pass,
//     Backend: BackendApi,
// {
//     type Target = P;
//     fn deref(&self) -> &Self::Target {
//         &self.pass
//     }
// }

// pub struct ImplRenderpass<Backend>
// where
//     Backend: BackendApi,
// {
//     pub data: Backend::Renderpass,
//     pub _m: PhantomData<Backend>,
// }
