extern crate ash;
extern crate tephra;
pub use tephra::winit;
#[cfg(windows)]
extern crate winapi;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use std::default::Default;
use std::ptr;
use tephra::failure::Fail;

use tephra::backend::vulkan::{record_submit_commandbuffer, Context};
use tephra::backend::BackendApi;
use tephra::buffer::{Buffer, BufferUsage};
use tephra::image::{Resolution, Image, RenderTarget, RenderTargetInfo};
use tephra::pipeline::{Pipeline, PipelineBuilder};
use tephra::renderpass::{Pass, Renderpass, VertexInput, VertexInputData, VertexType};
use tephra::shader::Shader;
use tephra::swapchain::Swapchain;

pub struct TrianglePass;
impl Pass for TrianglePass {
    type Input = Vertex;
    //type Output = Output;
}

pub struct Target<'a, Backend: BackendApi> {
    color: &'a Image<Backend>,
    depth: &'a Image<Backend>,
}

impl<'a, Backend: BackendApi> RenderTarget<Backend> for Target<'a, Backend> {
    fn render_target(&self) -> RenderTargetInfo<Backend> {
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
        let renderpass = Renderpass::new(&context, TrianglePass);
        let resolution = Resolution {
            width: 1920,
            height: 1080
        };
        //let swapchain = Swapchain::new(&context);
        let depth_image = Image::create_depth(&context, resolution);
        let framebuffers: Vec<vk::Framebuffer> = context
            .present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, context.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    render_pass: renderpass.impl_render_pass.data.render_pass,
                    attachment_count: framebuffer_attachments.len() as u32,
                    p_attachments: framebuffer_attachments.as_ptr(),
                    width: context.surface_resolution.width,
                    height: context.surface_resolution.height,
                    layers: 1,
                };
                context
                    .device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            }).collect();

        let index_buffer_data = [0u32, 1, 2];
        let index_buffer =
            Buffer::from_slice(&context, BufferUsage::Index.into(), &index_buffer_data)
                .expect("index buffer");

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

        let vertex_buffer = Buffer::from_slice(&context, BufferUsage::Vertex.into(), &vertices)
            .and_then(|buffer| buffer.copy_to_device_local())
            .expect("Failed to create vertex buffer");

        let vertex_shader_module =
            Shader::load(&context, "shader/triangle/vert.spv").expect("vertex");
        let fragment_shader_module =
            Shader::load(&context, "shader/triangle/frag.spv").expect("vertex");
        let pipeline = PipelineBuilder::new()
            .with_vertex_shader(vertex_shader_module)
            .with_fragment_shader(fragment_shader_module)
            .with_renderpass(&renderpass)
            .build(&context);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: context.surface_resolution.width as f32,
            height: context.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: context.surface_resolution.clone(),
        }];
        let graphic_pipeline = pipeline.data.pipeline;
        context.render_loop(|| {
            let present_index = context
                .swapchain_loader
                .acquire_next_image_khr(
                    context.swapchain,
                    std::u64::MAX,
                    context.present_complete_semaphore,
                    vk::Fence::null(),
                ).unwrap();
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
                render_pass: renderpass.impl_render_pass.data.render_pass,
                framebuffer: framebuffers[present_index as usize],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: context.surface_resolution.clone(),
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };
            record_submit_commandbuffer(
                &context.device,
                context.draw_command_buffer,
                &context.present_queue.inner,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[context.present_complete_semaphore],
                &[context.rendering_complete_semaphore],
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
                        &[vertex_buffer.impl_buffer.buffer.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        draw_command_buffer,
                        index_buffer.impl_buffer.buffer.buffer,
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
            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: &context.rendering_complete_semaphore,
                swapchain_count: 1,
                p_swapchains: &context.swapchain,
                p_image_indices: &present_index,
                p_results: ptr::null_mut(),
            };
            context
                .swapchain_loader
                .queue_present_khr(*context.present_queue.inner.lock(), &present_info)
                .unwrap();
        });

        context.device.device_wait_idle().unwrap();
        for framebuffer in framebuffers {
            context.device.destroy_framebuffer(framebuffer, None);
        }
    }
}
