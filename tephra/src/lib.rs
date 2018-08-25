#![feature(rust_2018_preview)]
extern crate parking_lot;
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
extern crate petgraph;
extern crate anymap;
pub mod swapchain;
pub mod backend;
pub mod buffer;
pub mod context;
pub mod image;
pub mod pipeline;
pub mod renderpass;
pub mod shader;
pub mod framegraph;
pub mod render;

#[derive(Copy, Clone, Default, Debug)]
pub struct Viewport {
    pub origin: (f32, f32),
    pub dimensions: (f32, f32),
    pub depth_range: (f32, f32),
}
