extern crate ash;
extern crate tephra;
pub use tephra::winit;
#[cfg(windows)]
extern crate winapi;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use std::default::Default;
use std::marker::PhantomData;
use std::ptr;
use tephra::failure::Fail;

use tephra::backend::vulkan::{self, Context};
use tephra::backend::BackendApi;
use tephra::buffer::{Buffer, BufferUsage, Property};
use tephra::context;
use tephra::framegraph::*;
use tephra::image::{Image, ImageDesc, ImageLayout, RenderTarget, RenderTargetInfo, Resolution};
use tephra::pipeline::PipelineState;
use tephra::renderpass::{VertexInput, VertexInputData, VertexType};
use tephra::shader::Shader;
use tephra::swapchain::{Swapchain, SwapchainError};

// pub struct TrianglePass;

// impl<'target> Pass<'target> for TrianglePass {
//     type Input = Vertex;
//     type Target = TriangleRT<'target>;
// }

// pub struct TriangleRT<'a> {
//     color: &'a Image,
//     depth: &'a Image,
// }

// impl<'a> RenderTarget<'a> for TriangleRT<'a> {
//     fn render_target(&self) -> RenderTargetInfo {
//         RenderTargetInfo {
//             image_views: vec![&self.color, &self.depth],
//         }
//     }
// }

#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl VertexInput for Vertex {
    fn vertex_input_data() -> Vec<VertexInputData> {
        vec![
            VertexInputData {
                binding: 0,
                location: 0,
                offset: 0,
                vertex_type: VertexType::F32(4),
            },
            VertexInputData {
                binding: 0,
                location: 1,
                offset: 4 * 32,
                vertex_type: VertexType::F32(4),
            },
        ]
    }
}
// pub fn gbuffer() {
//     let mut fg = Framegraph::new();
//     pub struct GBufferData {
//         depth_buffer: Resource<Image>,
//         gbuffer1: Resource<Image>,
//         gbuffer2: Resource<Image>,
//     }

//     let gbuffer_pass = fg.add_render_pass("GBuffer Pass", |builder| GBufferData {
//         depth_buffer: builder.create_image("Depth Buffer"),
//         gbuffer1: builder.create_image("GBuffer1"),
//         gbuffer2: builder.create_image("GBuffer2"),
//     });

//     pub struct LightingData {
//         depth_buffer: Resource<Image>,
//         gbuffer1: Resource<Image>,
//         gbuffer2: Resource<Image>,
//         lighting_buffer: Resource<Image>,
//     }

//     pub struct SomeOtherData {
//         gbuffer: Resource<Image>,
//     }

//     let some_other_pass = fg.add_render_pass("Some Other Pass", |builder| SomeOtherData {
//         gbuffer: builder.write(gbuffer_pass.gbuffer2),
//     });

//     let lighting_pass = fg.add_render_pass("Lighting Pass", |builder| LightingData {
//         depth_buffer: builder.read(gbuffer_pass.depth_buffer),
//         gbuffer1: builder.read(gbuffer_pass.gbuffer1),
//         gbuffer2: builder.read(some_other_pass.gbuffer),
//         lighting_buffer: builder.create_image("Lighting Buffer"),
//     });

//     pub struct PostData {
//         lighting_buffer: Resource<Image>,
//         color_image: Resource<Image>,
//     }

//     let post_pass = fg.add_render_pass("Postprocess Pass", |builder| PostData {
//         lighting_buffer: builder.read(lighting_pass.lighting_buffer),
//         color_image: builder.create_image("Color Image"),
//     });

//     let compiled_fg = fg.compile();
//     compiled_fg.export_graphviz("graph.dot");
// }

pub fn triangle_pass(ctx: &context::Context, resolution: Resolution) -> Framegraph<Compiled> {
    let mut fg = Framegraph::new();
    pub struct TriangleData {
        pub color: Resource<Image>,
        pub depth: Resource<Image>,
    }
    let triangle_pass = fg.add_render_pass(
        "Triangle Pass",
        |builder| {
            let color_desc = ImageDesc {
                layout: ImageLayout::Color,
                resolution,
            };
            let depth_desc = ImageDesc {
                layout: ImageLayout::Depth,
                resolution,
            };
            TriangleData {
                color: builder.create_image("Color", color_desc),
                depth: builder.create_image("Depth", depth_desc),
            }
        },
        // TODO: Infer framebuffer layout based on data,
        |data| vec![data.color, data.depth],
        |data, render, context| {
            let index_buffer_data = [0u32, 1, 2];
            let index_buffer = Buffer::from_slice(
                &context,
                Property::HostVisible,
                BufferUsage::Index,
                &index_buffer_data,
            ).expect("index buffer");

            let vertices = [
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.0, -1.0, 0.0, 1.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
            ];

            let vertex_buffer = Buffer::from_slice(
                &context,
                Property::HostVisible,
                BufferUsage::Vertex,
                &vertices,
            ).expect("Failed to create vertex buffer");
            // let vk_index = index_buffer.downcast::<Vulkan>();
            // let vk_vertex = vertex_buffer.downcast::<Vulkan>();

            let vertex_shader_module =
                Shader::load(&context, "shader/triangle/vert.spv").expect("vertex");
            let fragment_shader_module =
                Shader::load(&context, "shader/triangle/frag.spv").expect("vertex");
            let state = PipelineState::new()
                .with_vertex_shader(&vertex_shader_module)
                .with_fragment_shader(&fragment_shader_module);
            render.draw_indexed(&state, &vertex_buffer, &index_buffer);
        },
    );
    fg.compile(ctx)
}

fn main() {
    unsafe {
        let context = Context::new();
        let mut swapchain = Swapchain::new(&context);
        let triangle_pass = triangle_pass(&context, swapchain.resolution());
        loop {
            triangle_pass.execute(&context);
            std::thread::sleep_ms(2000);
        }
        //ctx.render_loop(|| {
        //    // let present_index = match swapchain.aquire_next_image() {
        //    //     Result::Ok(index) => index,
        //    //     Err(err) => match err {
        //    //         SwapchainError::OutOfDate => {
        //    //             swapchain.recreate();
        //    //             swapchain
        //    //                 .aquire_next_image()
        //    //                 .expect("Unable to acquire image")
        //    //         }
        //    //         _ => panic!("{}", err),
        //    //     },
        //    // };
        //    println!("---");
        //    triangle_pass.execute(&context);
        //    //swapchain.present(present_index);
        //    std::thread::sleep_ms(2000);
        //    println!("after wait");
        //});
    }
}
