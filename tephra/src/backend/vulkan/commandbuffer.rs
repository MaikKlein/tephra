use super::{Context, Vulkan};
use crate::commandbuffer::{self, Command, CommandList, Compute, Graphics, Submit, SubmitApi};
use crate::image::ImageHandle;
use ash::{version::DeviceV1_0, vk};
use std::ptr;

// #[derive(Copy, Clone)]
// struct Submission {
//     start: usize,
//     end: usize,
//     flags: vk::QueueFlags,
// }

// /// Determins on which queue a command can potentially run on
// fn queue_flags(command: &Command) -> vk::QueueFlags {
//     match command {
//         Command::CopyImage(_) => {
//             vk::QueueFlags::TRANSFER | vk::QueueFlags::COMPUTE | vk::QueueFlags::GRAPHICS
//         }
//         Command::Dispatch(_) => vk::QueueFlags::COMPUTE,
//         Command::Draw(_) => vk::QueueFlags::GRAPHICS,
//     }
// }

// fn generate_submissions<'cmd>(
//     commands: &'cmd [Command],
// ) -> impl Iterator<Item = Submission> + 'cmd {
//     use itertools::Itertools;
//     let points =
//         commands
//             .iter()
//             .enumerate()
//             .scan(vk::QueueFlags::all(), |curr_flags, (idx, cmd)| {
//                 let flags = *curr_flags & queue_flags(cmd);
//                 if flags.is_empty() {
//                     *curr_flags = flags;
//                     Some(idx)
//                 } else {
//                     None
//                 }
//             });
//     points.tuple_windows().map(move |(start, end)| {
//         let flags = commands[start..=end]
//             .iter()
//             .fold(vk::QueueFlags::all(), |flags, cmd| flags & queue_flags(cmd));
//         Submission { start, end, flags }
//     })
// }
struct PipelineBarrier {
    barrier: Barrier,
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    dependency_flags: vk::DependencyFlags,
}

enum Barrier {
    Memory(vk::MemoryBarrier),
    BufferMemory(vk::BufferMemoryBarrier),
    ImageMemory(vk::ImageMemoryBarrier),
    Execution,
}

enum Sync {
    PipelineBarrier(PipelineBarrier),
    Semaphore,
}

// fn stage_copy_image(
//     copy_image: &commandbuffer::CopyImage,
//     handle: ImageHandle,
// ) -> vk::PipelineStageFlags {
//     if handle == copy_image.src || handle == copy_image.dst {
//         return vk::PipelineStageFlags::TRANSFER;
//     }
//     panic!("Handle is not inside CopyImage")
// }

// fn access_copy_image(
//     copy_image: &commandbuffer::CopyImage,
//     handle: ImageHandle,
// ) -> vk::AccessFlags {
//     if handle == copy_image.src {
//         return vk::AccessFlags::TRANSFER_READ;
//     }
//     if handle == copy_image.dst {
//         return vk::AccessFlags::TRANSFER_WRITE;
//     }
//     panic!("Handle is not inside CopyImage")
// }

impl SubmitApi for Context {
    unsafe fn submit_commands(&self, commands: &CommandList) {
        let mut fences = Vec::new();
        let mut buffers = Vec::new();
        let device = &self.device;
        for submit in &commands.submits {
            let command_buffer = self.command_pool.get_command_buffer(self);
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            };
            device
                .begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
            for command in &submit.commands {
                match command {
                    Command::CopyImage(copy_image) => {
                        let src = self.images.get(copy_image.src);
                        let dst = self.images.get(copy_image.dst);
                        let aspect_mask = super::image::get_aspect_mask(&dst.desc);
                        let subresource_range = vk::ImageSubresourceRange {
                            aspect_mask,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        };
                        let sub_resource_layer = vk::ImageSubresourceLayers {
                            aspect_mask,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        };
                        let image_copy = vk::ImageCopy {
                            src_subresource: sub_resource_layer,
                            dst_subresource: sub_resource_layer,
                            src_offset: vk::Offset3D::default(),
                            dst_offset: vk::Offset3D::default(),
                            extent: vk::Extent3D {
                                width: dst.desc.resolution.width,
                                height: dst.desc.resolution.height,
                                depth: 1,
                            },
                        };
                        let src_barrier = vk::ImageMemoryBarrier {
                            src_access_mask: vk::AccessFlags::MEMORY_READ,
                            dst_access_mask: vk::AccessFlags::TRANSFER_READ,
                            old_layout: dst.layout,
                            new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            image: src.image,
                            subresource_range,
                            ..Default::default()
                        };
                        let dst_barrier = vk::ImageMemoryBarrier {
                            src_access_mask: vk::AccessFlags::empty(),
                            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                            old_layout: dst.layout,
                            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            image: dst.image,
                            subresource_range,
                            ..Default::default()
                        };
                        device.cmd_pipeline_barrier(
                            *command_buffer,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[src_barrier, dst_barrier],
                        );
                        device.cmd_copy_image(
                            *command_buffer,
                            src.image,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            dst.image,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            &[image_copy],
                        );
                        let src_barrier = vk::ImageMemoryBarrier {
                            src_access_mask: vk::AccessFlags::TRANSFER_READ,
                            dst_access_mask: vk::AccessFlags::MEMORY_READ,
                            new_layout: src.layout,
                            old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            image: src.image,
                            subresource_range,
                            ..Default::default()
                        };
                        let dst_barrier = vk::ImageMemoryBarrier {
                            src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                            dst_access_mask: vk::AccessFlags::MEMORY_READ,
                            new_layout: dst.layout,
                            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            image: dst.image,
                            subresource_range,
                            ..Default::default()
                        };
                        device.cmd_pipeline_barrier(
                            *command_buffer,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[src_barrier, dst_barrier],
                        );
                    }
                    Command::Dispatch(dispatch) => {
                        let pipeline = self.compute_pipelines.get(dispatch.pipeline);
                        let descriptor = self.descriptors.get(dispatch.shader_arguments);
                        device.cmd_bind_pipeline(
                            *command_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            pipeline.pipeline,
                        );
                        device.cmd_bind_descriptor_sets(
                            *command_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            pipeline.layout,
                            0,
                            &[descriptor.descriptor_set],
                            &[],
                        );
                        device.cmd_dispatch(*command_buffer, dispatch.x, dispatch.y, dispatch.z);
                    }
                    Command::Draw(draw) => {
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

                        let viewports = [vk::Viewport {
                            x: 0.0,
                            y: 0.0,
                            width: self.surface_resolution.width as f32,
                            height: self.surface_resolution.height as f32,
                            min_depth: 0.0,
                            max_depth: 1.0,
                        }];
                        let scissors = [vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: self.surface_resolution.clone(),
                        }];
                        let framebuffer = self.framebuffers.get(draw.framebuffer);
                        let pipeline = self.graphic_pipelines.get(draw.graphics_pipeline);
                        let vertex_buffer = self.buffers.get(draw.vertex);
                        let index_buffer = self.buffers.get(draw.index.buffer);
                        let renderpass = self.renderpasses.get(draw.renderpass);
                        let descriptor = self.descriptors.get(draw.shader_arguments);
                        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                            .render_pass(renderpass.render_pass)
                            .framebuffer(framebuffer.framebuffer)
                            .render_area(vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: self.surface_resolution.clone(),
                            })
                            .clear_values(&clear_values);
                        device.cmd_begin_render_pass(
                            *command_buffer,
                            &render_pass_begin_info,
                            vk::SubpassContents::INLINE,
                        );

                        // TODO: Make configurable
                        device.cmd_set_viewport(*command_buffer, 0, &viewports);
                        device.cmd_set_scissor(*command_buffer, 0, &scissors);

                        device.cmd_bind_pipeline(
                            *command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline.pipeline,
                        );
                        device.cmd_bind_descriptor_sets(
                            *command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline.layout,
                            0,
                            &[descriptor.descriptor_set],
                            &[],
                        );
                        device.cmd_bind_vertex_buffers(
                            *command_buffer,
                            0,
                            &[vertex_buffer.buffer],
                            &[0],
                        );
                        device.cmd_bind_index_buffer(
                            *command_buffer,
                            index_buffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_end_render_pass(*command_buffer);
                    }
                }
            }
            device
                .end_command_buffer(*command_buffer)
                .expect("End commandbuffer");
            let fence_info = vk::FenceCreateInfo::default();
            let fence = device.create_fence(&fence_info, None).unwrap();
            let queue = self.present_queue.inner.lock();

            // TODO: Properly implement submission with batching and semaphores
            device
                .queue_submit(
                    *queue,
                    &[vk::SubmitInfo::builder()
                        .command_buffers(&[*command_buffer])
                        .build()],
                    fence,
                )
                .unwrap();
            device
                .wait_for_fences(&[fence], true, u64::max_value())
                .unwrap();
            fences.push(fence);
            buffers.push(command_buffer);
        }

        for fence in fences {
            device.destroy_fence(fence, None);
        }
        for cmd_buffer in buffers {
            cmd_buffer.release();
        }
    }
}

// pub struct Commandbuffer {
//     ctx: Context,
// }

// // impl CommandbufferApi for Commandbuffer<Graphics> {
// // }
// pub struct Execute {
//     ctx: Context,
// }
// impl ExecuteApi for Execute {
//     fn execute_commands(&self, cmds: &[GraphicsCmd]) {
//         let _clear_values = [
//             vk::ClearValue {
//                 color: vk::ClearColorValue {
//                     float32: [0.0, 0.0, 0.0, 0.0],
//                 },
//             },
//             vk::ClearValue {
//                 depth_stencil: vk::ClearDepthStencilValue {
//                     depth: 1.0,
//                     stencil: 0,
//                 },
//             },
//         ];
//         //let mut pipelines = Vec::new();
//         super::CommandBuffer::record(&self.ctx, "Execute", |_cb| {
//             let _device = &self.ctx.device;
//             let mut _outer_render: Option<&super::render::Render> = None;
//             for cmd in cmds {
//                 match cmd {
//                     GraphicsCmd::BindPipeline {
//                         state: _,
//                         stride: _,
//                         vertex_input_data: _,
//                     } => {
//                         unsafe {
//                             // let pipeline = super::render::create_pipeline(
//                             //     &self.ctx,
//                             //     state,
//                             //     *stride,
//                             //     &vertex_input_data,
//                             //     outer_render.unwrap().renderpass,
//                             //     );
//                             // pipelines.push(pipeline);
//                         }
//                     }
//                     _ => (),
//                 }
//             }
//         });
//     }
// }
// impl CreateExecute for Context {
//     fn create_execute(&self) -> commandbuffer::Execute {
//         let execute = Execute { ctx: self.clone() };
//         commandbuffer::Execute {
//             inner: Box::new(execute),
//         }
//     }
// }
