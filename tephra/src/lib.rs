#![feature(rust_2018_preview)]
//#![feature(nll)]
extern crate parking_lot;
extern crate rspirv;
extern crate spirv_headers;
extern crate thread_local_object;
#[macro_use]
pub extern crate ash;
pub extern crate failure;
extern crate serde;
pub extern crate winit;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate downcast_rs as downcast;
extern crate anymap;
extern crate petgraph;
pub mod backend;
pub mod buffer;
pub mod commandbuffer;
pub mod context;
pub mod descriptor;
pub mod framegraph;
pub mod image;
pub mod pipeline;
pub mod reflect;
pub mod render;
pub mod renderpass;
pub mod shader;
pub mod swapchain;
#[derive(Copy, Clone, Default, Debug)]
pub struct Viewport {
    pub origin: (f32, f32),
    pub dimensions: (f32, f32),
    pub depth_range: (f32, f32),
}
