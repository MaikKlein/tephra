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

use tephra::backend::vulkan::{self, record_submit_commandbuffer, Context, Vulkan};
use tephra::backend::BackendApi;
use tephra::buffer::{Buffer, BufferUsage, Property};
use tephra::image::{Image, RenderTarget, RenderTargetInfo, Resolution};
use tephra::pipeline::{Pipeline, PipelineBuilder};
use tephra::renderpass::{Pass, Renderpass, VertexInput, VertexInputData, VertexType};
use tephra::shader::Shader;
use tephra::swapchain::Swapchain;

pub struct TrianglePass<'a> {
    _m: PhantomData<&'a ()>,
}
impl<'a> TrianglePass<'a> {
    pub fn new() -> Self {
        TrianglePass { _m: PhantomData }
    }
}

impl<'a> Pass for TrianglePass<'a> {
    type Input = Vertex;
    type Target = Target<'a>;
}

pub struct Target<'a> {
    color: &'a Image,
    depth: &'a Image,
}

impl<'a> RenderTarget for Target<'a> {
    fn render_target(&self) -> RenderTargetInfo {
        RenderTargetInfo {
            image_views: vec![&self.color, &self.depth],
        }
    }
}

#[derive(Clone, Debug, Copy)]
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
                vertex_type: VertexType::F32(4),
            },
            VertexInputData {
                binding: 0,
                location: 1,
                vertex_type: VertexType::F32(4),
            },
        ]
    }
}

fn main() {
    unsafe {
        let context = Context::new();
        let ctx = context.context.downcast_ref::<Context>().unwrap();
        let renderpass = Renderpass::new(&context, TrianglePass::new());
        let vkrenderpass = renderpass
            .renderpass
            .downcast_ref::<vulkan::renderpass::RenderpassData>()
            .unwrap()
            .renderpass;
        let resolution = Resolution {
            width: 1920,
            height: 1080,
        };
        let swapchain = Swapchain::new(&context);
        let depth_image = Image::create_depth(&context, resolution);
        let framebuffers: Vec<vk::Framebuffer> = swapchain.present_images()
            .iter()
            .map(|present_image| {
                let present_image_view = present_image.downcast::<Vulkan>().image_view;
                let framebuffer_attachments = [present_image_view, ctx.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    render_pass: vkrenderpass,
                    attachment_count: framebuffer_attachments.len() as u32,
                    p_attachments: framebuffer_attachments.as_ptr(),
                    width: ctx.surface_resolution.width,
                    height: ctx.surface_resolution.height,
                    layers: 1,
                };
                ctx.device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            }).collect();

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
        let vk_index = index_buffer.downcast::<Vulkan>();
        let vk_vertex = vertex_buffer.downcast::<Vulkan>();

        let vertex_shader_module =
            Shader::load(&context, "shader/triangle/vert.spv").expect("vertex");
        let fragment_shader_module =
            Shader::load(&context, "shader/triangle/frag.spv").expect("vertex");
        let pipeline = PipelineBuilder::new()
            .with_vertex_shader(&vertex_shader_module)
            .with_fragment_shader(&fragment_shader_module)
            .with_renderpass(&renderpass)
            .build(&context);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: ctx.surface_resolution.width as f32,
            height: ctx.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: ctx.surface_resolution.clone(),
        }];
        let graphic_pipeline = pipeline.downcast::<Vulkan>().pipeline;
        ctx.render_loop(|| {
            let present_index = swapchain.aquire_next_image();
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass: vkrenderpass,
                framebuffer: framebuffers[present_index as usize],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: ctx.surface_resolution.clone(),
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };
            record_submit_commandbuffer(
                &ctx.device,
                ctx.draw_command_buffer,
                &ctx.present_queue.inner,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[ctx.present_complete_semaphore],
                &[ctx.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        graphic_pipeline,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_vertex_buffers(
                        draw_command_buffer,
                        0,
                        &[vk_vertex.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        draw_command_buffer,
                        vk_index.buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_draw_indexed(
                        draw_command_buffer,
                        index_buffer_data.len() as u32,
                        1,
                        0,
                        0,
                        1,
                    );
                    // Or draw without the index buffer
                    // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
            //let mut present_info_err = mem::uninitialized();
            swapchain.present(present_index);
        });

        // context.device.device_wait_idle().unwrap();
        // for framebuffer in framebuffers {
        //     context.device.destroy_framebuffer(framebuffer, None);
        // }
    }
}
