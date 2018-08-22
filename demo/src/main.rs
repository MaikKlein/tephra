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

            // let vertices = [
            //     Vertex {
            //         pos: [-1.0, 1.0, 0.0, 1.0],
            //         color: [0.0, 1.0, 0.0, 1.0],
            //     },
            //     Vertex {
            //         pos: [1.0, 1.0, 0.0, 1.0],
            //         color: [0.0, 0.0, 1.0, 1.0],
            //     },
            //     Vertex {
            //         pos: [0.0, -1.0, 0.0, 1.0],
            //         color: [1.0, 0.0, 0.0, 1.0],
            //     },
            // ];

            // let vertex_buffer = Buffer::from_slice(
            //     &context,
            //     Property::HostVisible,
            //     BufferUsage::Vertex,
            //     &vertices,
            // ).expect("Failed to create vertex buffer");
            // let vk_index = index_buffer.downcast::<Vulkan>();
            // let vk_vertex = vertex_buffer.downcast::<Vulkan>();

            // let vertex_shader_module =
            //     Shader::load(&context, "shader/triangle/vert.spv").expect("vertex");
            // let fragment_shader_module =
            //     Shader::load(&context, "shader/triangle/frag.spv").expect("vertex");
            // let state = PipelineState::new()
            //     .with_vertex_shader(&vertex_shader_module)
            //     .with_fragment_shader(&fragment_shader_module);
            // render.draw_indexed(&state, &vertex_buffer, &index_buffer);
        },
    );
    fg.compile(ctx)
}

fn main() {
    unsafe {
        //gbuffer();
        let context = Context::new();
        let ctx = context.context.downcast_ref::<Context>().unwrap();
        // let renderpass = Renderpass::new(&context, TrianglePass {});
        // let vkrenderpass = renderpass
        //     .renderpass
        //     .downcast_ref::<vulkan::renderpass::RenderpassData>()
        //     .unwrap()
        //     .renderpass;
        let mut swapchain = Swapchain::new(&context);
        let triangle_pass = triangle_pass(&context, swapchain.resolution());
        // let framebuffers: Vec<vk::Framebuffer> = swapchain
        //     .present_images()
        //     .iter()
        //     .map(|present_image| {
        //         let present_image_view = present_image.downcast::<Vulkan>().image_view;
        //         let framebuffer_attachments = [present_image_view, ctx.depth_image_view];
        //         let frame_buffer_create_info = vk::FramebufferCreateInfo {
        //             s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        //             p_next: ptr::null(),
        //             flags: Default::default(),
        //             render_pass: vkrenderpass,
        //             attachment_count: framebuffer_attachments.len() as u32,
        //             p_attachments: framebuffer_attachments.as_ptr(),
        //             width: ctx.surface_resolution.width,
        //             height: ctx.surface_resolution.height,
        //             layers: 1,
        //         };
        //         ctx.device
        //             .create_framebuffer(&frame_buffer_create_info, None)
        //             .unwrap()
        //     }).collect();

        // let viewports = [vk::Viewport {
        //     x: 0.0,
        //     y: 0.0,
        //     width: ctx.surface_resolution.width as f32,
        //     height: ctx.surface_resolution.height as f32,
        //     min_depth: 0.0,
        //     max_depth: 1.0,
        // }];
        // let scissors = [vk::Rect2D {
        //     offset: vk::Offset2D { x: 0, y: 0 },
        //     extent: ctx.surface_resolution.clone(),
        // }];
        // let graphic_pipeline = pipeline.downcast::<Vulkan>().pipeline;
        ctx.render_loop(|| {
            // let present_index = match swapchain.aquire_next_image() {
            //     Result::Ok(index) => index,
            //     Err(err) => match err {
            //         SwapchainError::OutOfDate => {
            //             swapchain.recreate();
            //             swapchain
            //                 .aquire_next_image()
            //                 .expect("Unable to acquire image")
            //         }
            //         _ => panic!("{}", err),
            //     },
            // };
            triangle_pass.execute(&context);
            println!("swapchain {:?}", swapchain.resolution());
            // let clear_values = [
            //     vk::ClearValue {
            //         color: vk::ClearColorValue {
            //             float32: [0.0, 0.0, 0.0, 0.0],
            //         },
            //     },
            //     vk::ClearValue {
            //         depth_stencil: vk::ClearDepthStencilValue {
            //             depth: 1.0,
            //             stencil: 0,
            //         },
            //     },
            // ];

            // let render_pass_begin_info = vk::RenderPassBeginInfo {
            //     s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            //     p_next: ptr::null(),
            //     render_pass: vkrenderpass,
            //     framebuffer: framebuffers[present_index as usize],
            //     render_area: vk::Rect2D {
            //         offset: vk::Offset2D { x: 0, y: 0 },
            //         extent: ctx.surface_resolution.clone(),
            //     },
            //     clear_value_count: clear_values.len() as u32,
            //     p_clear_values: clear_values.as_ptr(),
            // };
            // record_submit_commandbuffer(
            //     &ctx.device,
            //     ctx.draw_command_buffer,
            //     &ctx.present_queue.inner,
            //     &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            //     &[ctx.present_complete_semaphore],
            //     &[ctx.rendering_complete_semaphore],
            //     |device, draw_command_buffer| {
            //         device.cmd_begin_render_pass(
            //             draw_command_buffer,
            //             &render_pass_begin_info,
            //             vk::SubpassContents::INLINE,
            //         );
            //         device.cmd_bind_pipeline(
            //             draw_command_buffer,
            //             vk::PipelineBindPoint::GRAPHICS,
            //             graphic_pipeline,
            //         );
            //         device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
            //         device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
            //         device.cmd_bind_vertex_buffers(
            //             draw_command_buffer,
            //             0,
            //             &[vk_vertex.buffer],
            //             &[0],
            //         );
            //         device.cmd_bind_index_buffer(
            //             draw_command_buffer,
            //             vk_index.buffer,
            //             0,
            //             vk::IndexType::UINT32,
            //         );
            //         device.cmd_draw_indexed(
            //             draw_command_buffer,
            //             index_buffer_data.len() as u32,
            //             1,
            //             0,
            //             0,
            //             1,
            //         );
            //         // Or draw without the index buffer
            //         // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
            //         device.cmd_end_render_pass(draw_command_buffer);
            //     },
            // );
            //swapchain.present(present_index);
            std::thread::sleep_ms(2000);
        });
    }
}
