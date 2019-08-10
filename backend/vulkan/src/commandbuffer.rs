use super::Context;
use ash::{version::DeviceV1_0, vk};
use std::collections::HashMap;
use std::ptr;
use tephra::{
    commandbuffer::{self, Command, CommandList, SubmitApi},
    descriptor::Pool,
};
use vk_sync::AccessType;

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

impl SubmitApi for Context {
    unsafe fn submit_commands(&self, pool: &mut Pool, commands: &CommandList) {
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
                        device.cmd_bind_pipeline(
                            *command_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            pipeline.pipeline,
                        );
                        for (set, shader_arguments) in dispatch.shader_arguments.iter() {
                            let descriptor_handle = pool.allocate(shader_arguments);
                            let descriptor = self.descriptors.get(descriptor_handle);
                            device.cmd_bind_descriptor_sets(
                                *command_buffer,
                                vk::PipelineBindPoint::COMPUTE,
                                pipeline.layout,
                                *set,
                                &[descriptor.descriptor_set],
                                &[],
                            );
                        }
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
                        for (set, shader_arguments) in draw.shader_arguments.iter() {
                            let descriptor_handle = pool.allocate(shader_arguments);
                            let descriptor = self.descriptors.get(descriptor_handle);
                            device.cmd_bind_descriptor_sets(
                                *command_buffer,
                                vk::PipelineBindPoint::GRAPHICS,
                                pipeline.layout,
                                *set,
                                &[descriptor.descriptor_set],
                                &[],
                            );
                        }
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
                        let index_len = index_buffer.size / std::mem::size_of::<u32>() as u64;
                        device.cmd_draw_indexed(*command_buffer, index_len as u32, 1, 0, 0, 1);
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
