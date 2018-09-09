use super::buffer::BufferData;
use super::Context;
use super::{CommandBuffer, Vulkan};
use ash::version::DeviceV1_0;
use ash::vk;
use buffer::BufferApi;
use commandbuffer::{ComputeCmd, GraphicsCmd};
use descriptor::NativeLayout;
use framegraph::{Compiled, Framegraph};
use image::{Image, ImageLayout, Resolution};
use pipeline::PipelineState;
use render::{self, ComputeApi, CreateCompute, CreateRender, RenderApi};
use renderpass::{VertexInputData, VertexType};
use std::ffi::CString;
use std::ptr;

pub struct Compute {
    pub ctx: Context,
    pub pipeline_layout: vk::PipelineLayout,
}
impl ComputeApi for Compute {
    fn execute_commands(&self, cmds: &[ComputeCmd]) {}
}
impl CreateCompute for Context {
    fn create_compute(&self, layout: &NativeLayout) -> render::Compute {
        let pipeline_layout = unsafe { create_pipeline_layout(self, layout) };
        let inner = Compute {
            ctx: self.clone(),
            pipeline_layout,
        };
        render::Compute {
            inner: Box::new(inner),
        }
    }
}
pub struct Render {
    pub ctx: Context,
    pub framebuffer: vk::Framebuffer,
    pub renderpass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub surface_resolution: Resolution,
}

impl RenderApi for Render {
    fn execute_commands(&self, cmds: &[GraphicsCmd]) {
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.ctx.surface_resolution.width as f32,
            height: self.ctx.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.ctx.surface_resolution.clone(),
        }];
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
        let mut pipelines = Vec::new();
        let command_buffer =
            CommandBuffer::record(&self.ctx, "RenderPass", |draw_command_buffer| {
                let device = &self.ctx.device;
                unsafe {
                    let render_pass_begin_info = vk::RenderPassBeginInfo {
                        s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                        p_next: ptr::null(),
                        render_pass: self.renderpass,
                        framebuffer: self.framebuffer,
                        render_area: vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: self.ctx.surface_resolution.clone(),
                        },
                        clear_value_count: clear_values.len() as u32,
                        p_clear_values: clear_values.as_ptr(),
                    };
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    for cmd in cmds {
                        match cmd {
                            GraphicsCmd::BindDescriptor(descriptor) => {
                                let vk_descriptor = descriptor.inner.as_ref().downcast::<Vulkan>();
                                device.cmd_bind_descriptor_sets(
                                    draw_command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    self.pipeline_layout,
                                    0,
                                    &[vk_descriptor.descriptor_set],
                                    &[],
                                );
                            }
                            GraphicsCmd::BindVertex(buffer) => {
                                let vk_vertex_buffer = buffer.as_ref().downcast::<Vulkan>();

                                device.cmd_bind_vertex_buffers(
                                    draw_command_buffer,
                                    0,
                                    &[vk_vertex_buffer.buffer],
                                    &[0],
                                );
                            }
                            GraphicsCmd::BindIndex(buffer) => {
                                let vk_index_buffer = buffer.as_ref().downcast::<Vulkan>();
                                device.cmd_bind_index_buffer(
                                    draw_command_buffer,
                                    vk_index_buffer.buffer,
                                    0,
                                    vk::IndexType::UINT32,
                                );
                            }
                            GraphicsCmd::BindPipeline {
                                state,
                                stride,
                                ref vertex_input_data,
                            } => {
                                let pipeline = create_pipeline(
                                    &self.ctx,
                                    state,
                                    *stride,
                                    vertex_input_data,
                                    self.renderpass,
                                    self.pipeline_layout,
                                );
                                pipelines.push(pipeline);
                                device.cmd_bind_pipeline(
                                    draw_command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    pipeline,
                                );
                            }
                            GraphicsCmd::DrawIndex { len } => {
                                device.cmd_draw_indexed(draw_command_buffer, *len, 1, 0, 0, 1);
                            }
                        }
                    }
                    device.cmd_end_render_pass(draw_command_buffer);
                }
            });
        self.ctx.present_queue.submit(
            &self.ctx,
            &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
            // FIXME: Add proper sync points
            &[],
            &[],
            command_buffer,
        );
    }
    // fn draw_indexed(
    //     &self,
    //     state: &PipelineState,
    //     stride: u32,
    //     vertex_input: &[VertexInputData],
    //     vertex_buffer: &BufferApi,
    //     index_buffer: &BufferApi,
    //     len: u32,
    // ) {
    //     let vk_vertex_buffer = vertex_buffer.downcast_ref::<BufferData>().expect("backend");
    //     let vk_index_buffer = index_buffer.downcast_ref::<BufferData>().expect("backend");
    //     let pipeline = unsafe {
    //         create_pipeline(
    //             &self.ctx,
    //             state,
    //             stride,
    //             vertex_input,
    //             self.renderpass,
    //             self.pipeline_layout,
    //         )
    //     };
    //     let ctx = &self.ctx;
    //     let viewports = [vk::Viewport {
    //         x: 0.0,
    //         y: 0.0,
    //         width: ctx.surface_resolution.width as f32,
    //         height: ctx.surface_resolution.height as f32,
    //         min_depth: 0.0,
    //         max_depth: 1.0,
    //     }];
    //     let scissors = [vk::Rect2D {
    //         offset: vk::Offset2D { x: 0, y: 0 },
    //         extent: ctx.surface_resolution.clone(),
    //     }];
    //     let clear_values = [
    //         vk::ClearValue {
    //             color: vk::ClearColorValue {
    //                 float32: [0.0, 0.0, 0.0, 0.0],
    //             },
    //         },
    //         vk::ClearValue {
    //             depth_stencil: vk::ClearDepthStencilValue {
    //                 depth: 1.0,
    //                 stencil: 0,
    //             },
    //         },
    //     ];
    //     let command_buffer = CommandBuffer::record(ctx, "RenderPass", |draw_command_buffer| {
    //         let device = &ctx.device;
    //         unsafe {
    //             let render_pass_begin_info = vk::RenderPassBeginInfo {
    //                 s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
    //                 p_next: ptr::null(),
    //                 render_pass: self.renderpass,
    //                 framebuffer: self.framebuffer,
    //                 render_area: vk::Rect2D {
    //                     offset: vk::Offset2D { x: 0, y: 0 },
    //                     extent: ctx.surface_resolution.clone(),
    //                 },
    //                 clear_value_count: clear_values.len() as u32,
    //                 p_clear_values: clear_values.as_ptr(),
    //             };
    //             device.cmd_begin_render_pass(
    //                 draw_command_buffer,
    //                 &render_pass_begin_info,
    //                 vk::SubpassContents::INLINE,
    //             );
    //             device.cmd_bind_pipeline(
    //                 draw_command_buffer,
    //                 vk::PipelineBindPoint::GRAPHICS,
    //                 pipeline,
    //             );
    //             device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
    //             device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
    //             device.cmd_bind_vertex_buffers(
    //                 draw_command_buffer,
    //                 0,
    //                 &[vk_vertex_buffer.buffer],
    //                 &[0],
    //             );
    //             device.cmd_bind_index_buffer(
    //                 draw_command_buffer,
    //                 vk_index_buffer.buffer,
    //                 0,
    //                 vk::IndexType::UINT32,
    //             );
    //             device.cmd_draw_indexed(draw_command_buffer, len, 1, 0, 0, 1);
    //             // Or draw without the index buffer
    //             // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
    //             device.cmd_end_render_pass(draw_command_buffer);
    //         }
    //     });
    //     ctx.present_queue.submit(
    //         ctx,
    //         &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
    //         // FIXME: Add proper sync points
    //         &[],
    //         &[],
    //         command_buffer,
    //     );
    // }
}

impl CreateRender for Context {
    fn create_render(
        &self,
        resolution: Resolution,
        images: &[&Image],
        layout: &NativeLayout,
    ) -> render::Render {
        unsafe {
            let renderpass = create_renderpass(self, images);
            let framebuffer = create_framebuffer(self, renderpass, images);
            let pipeline_layout = create_pipeline_layout(self, layout);
            let ctx = self.clone();
            let render = Render {
                renderpass,
                framebuffer,
                pipeline_layout,
                surface_resolution: resolution,
                ctx,
            };
            render::Render {
                inner: Box::new(render),
            }
        }
    }
}

fn create_framebuffer(
    ctx: &Context,
    renderpass: vk::RenderPass,
    image_resources: &[&Image],
) -> vk::Framebuffer {
    let framebuffer_attachments: Vec<_> = image_resources
        .iter()
        .map(|image| image.downcast::<Vulkan>().image_view)
        .collect();
    let frame_buffer_create_info = vk::FramebufferCreateInfo {
        s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        render_pass: renderpass,
        attachment_count: framebuffer_attachments.len() as u32,
        p_attachments: framebuffer_attachments.as_ptr(),
        width: ctx.surface_resolution.width,
        height: ctx.surface_resolution.height,
        layers: 1,
    };
    unsafe {
        ctx.device
            .create_framebuffer(&frame_buffer_create_info, None)
            .unwrap()
    }
}
pub unsafe fn create_pipeline_layout(ctx: &Context, layout: &NativeLayout) -> vk::PipelineLayout {
    let vk_layout = layout.inner.as_ref().downcast::<Vulkan>();
    let layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        set_layout_count: vk_layout.layouts.len() as _,
        p_set_layouts: vk_layout.layouts.as_ptr(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
    };

    ctx.device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
}
pub unsafe fn create_pipeline(
    ctx: &Context,
    state: &PipelineState,
    stride: u32,
    _vertex_input: &[VertexInputData],
    renderpass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
) -> vk::Pipeline {
    let vertex_shader = state.vertex_shader.as_ref().expect("vertex");
    let vk_vertex = vertex_shader.downcast::<Vulkan>();
    let fragment_shader = state.fragment_shader.as_ref().expect("vertex");
    let vk_fragment = fragment_shader.downcast::<Vulkan>();

    let shader_entry_name = CString::new("main").unwrap();
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            module: vk_vertex.shader_module,
            p_name: shader_entry_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::VERTEX,
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            module: vk_fragment.shader_module,
            p_name: shader_entry_name.as_ptr(),
            p_specialization_info: ptr::null(),
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];
    let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
        binding: 0,
        stride,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    let vertex_input_attribute_descriptions = vertex_input(_vertex_input);
    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
        p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
        vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
        p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
    };
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        flags: Default::default(),
        p_next: ptr::null(),
        primitive_restart_enable: 0,
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
    };
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
    let viewport_state_info = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        scissor_count: scissors.len() as u32,
        p_scissors: scissors.as_ptr(),
        viewport_count: viewports.len() as u32,
        p_viewports: viewports.as_ptr(),
    };
    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        cull_mode: vk::CullModeFlags::NONE,
        depth_bias_clamp: 0.0,
        depth_bias_constant_factor: 0.0,
        depth_bias_enable: 0,
        depth_bias_slope_factor: 0.0,
        depth_clamp_enable: 0,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        rasterizer_discard_enable: 0,
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        flags: Default::default(),
        p_next: ptr::null(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: 0,
        min_sample_shading: 0.0,
        p_sample_mask: ptr::null(),
        alpha_to_one_enable: 0,
        alpha_to_coverage_enable: 0,
    };
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    };
    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: noop_stencil_state.clone(),
        back: noop_stencil_state.clone(),
        max_depth_bounds: 1.0,
        min_depth_bounds: 0.0,
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::ZERO,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::all(),
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: color_blend_attachment_states.len() as u32,
        p_attachments: color_blend_attachment_states.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0],
    };
    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags: Default::default(),
        dynamic_state_count: dynamic_state.len() as u32,
        p_dynamic_states: dynamic_state.as_ptr(),
    };
    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: shader_stage_create_infos.len() as u32,
        p_stages: shader_stage_create_infos.as_ptr(),
        p_vertex_input_state: &vertex_input_state_info,
        p_input_assembly_state: &vertex_input_assembly_state_info,
        p_tessellation_state: ptr::null(),
        p_viewport_state: &viewport_state_info,
        p_rasterization_state: &rasterization_info,
        p_multisample_state: &multisample_state_info,
        p_depth_stencil_state: &depth_state_info,
        p_color_blend_state: &color_blend_state,
        p_dynamic_state: &dynamic_state_info,
        layout: pipeline_layout,
        render_pass: renderpass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    };
    let graphics_pipelines = ctx
        .device
        .create_graphics_pipelines(ctx.pipeline_cache, &[graphic_pipeline_info], None)
        .expect("Unable to create graphics pipeline");
    graphics_pipelines[0]
}
unsafe fn create_renderpass(ctx: &Context, image_resources: &[&Image]) -> vk::RenderPass {
    let renderpass_attachments: Vec<_> = image_resources
        .iter()
        .map(|image| match image.desc().layout {
            ImageLayout::Color => vk::AttachmentDescription {
                format: vk::Format::R8G8B8A8_UNORM,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            ImageLayout::Depth => vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
        })
        .collect();

    let color_attachments: Vec<_> = image_resources
        .iter()
        .enumerate()
        .filter_map(|(idx, image)| match image.desc().layout {
            ImageLayout::Color => {
                let color_attachment_ref = vk::AttachmentReference {
                    attachment: idx as _,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                };
                Some(color_attachment_ref)
            }
            _ => None,
        })
        .collect();
    let depth_attachments: Vec<_> = image_resources
        .iter()
        .enumerate()
        .filter_map(|(idx, image)| match image.desc().layout {
            ImageLayout::Depth => {
                let depth_attachment_ref = vk::AttachmentReference {
                    attachment: idx as _,
                    layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                };
                Some(depth_attachment_ref)
            }
            _ => None,
        })
        .collect();

    let dependency = vk::SubpassDependency {
        dependency_flags: Default::default(),
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: Default::default(),
        src_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    };
    let subpass = vk::SubpassDescription {
        color_attachment_count: color_attachments.len() as _,
        p_color_attachments: color_attachments.as_ptr(),
        p_depth_stencil_attachment: depth_attachments.as_ptr(),
        flags: Default::default(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: ptr::null(),
        p_resolve_attachments: ptr::null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: ptr::null(),
    };
    let renderpass_create_info = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        flags: Default::default(),
        p_next: ptr::null(),
        attachment_count: renderpass_attachments.len() as u32,
        p_attachments: renderpass_attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &dependency,
    };
    ctx.device
        .create_render_pass(&renderpass_create_info, None)
        .unwrap()
}

pub fn vertex_format(ty: VertexType) -> vk::Format {
    match ty {
        VertexType::F32(size) => match size {
            1 => vk::Format::R32_SFLOAT,
            2 => vk::Format::R32G32_SFLOAT,
            3 => vk::Format::R32G32B32_SFLOAT,
            4 => vk::Format::R32G32B32A32_SFLOAT,
            _ => unreachable!(),
        },
    }
}
pub fn vertex_input(vertex_input: &[VertexInputData]) -> Vec<vk::VertexInputAttributeDescription> {
    vertex_input
        .iter()
        .map(|input| vk::VertexInputAttributeDescription {
            location: input.location,
            binding: input.binding,
            offset: input.offset,
            format: vertex_format(input.vertex_type),
        })
        .collect()
}
