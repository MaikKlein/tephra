use super::image::from_format;
use super::Context;
use tephra::{
    image::Image,
    renderpass::{
        Attachment, Framebuffer, FramebufferApi, Renderpass, RenderpassApi, RenderpassState,
    },
};
use ash::{version::DeviceV1_0, vk};
pub struct FramebufferData {
    pub framebuffer: vk::Framebuffer,
}
impl FramebufferApi for Context {
    unsafe fn create_framebuffer(
        &self,
        renderpass: Renderpass,
        images: &[Image],
    ) -> Framebuffer {
        let renderpass_data = self.renderpasses.get(renderpass);
        // TODO: Proper resolution
        let framebuffer_attachments: Vec<_> = images
            .iter()
            .map(|&image| self.images.get(image.handle).image_view)
            .collect();
        let frame_buffer_create_info = vk::FramebufferCreateInfo {
            render_pass: renderpass_data.render_pass,
            attachment_count: framebuffer_attachments.len() as u32,
            p_attachments: framebuffer_attachments.as_ptr(),
            width: self.surface_resolution.width,
            height: self.surface_resolution.height,
            layers: 1,
            ..Default::default()
        };
        let framebuffer = self
            .device
            .create_framebuffer(&frame_buffer_create_info, None)
            .unwrap();
        let data = FramebufferData { framebuffer };
        self.framebuffers.insert(data)
    }
}
pub struct RenderpassData {
    pub render_pass: vk::RenderPass,
}
impl RenderpassApi for Context {
    unsafe fn create_renderpass(
        &self,
        builder: &RenderpassState,
    ) -> Renderpass {
        fn build_attachment(
            ctx: &Context,
            attachment: &Attachment,
            layout: vk::ImageLayout,
        ) -> vk::AttachmentDescription {
            // TODO: Add more configs
            vk::AttachmentDescription {
                format: from_format(attachment.format),
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: layout,
            }
        }
        let mut attachments: Vec<_> = builder
            .color_attachments
            .iter()
            .map(|attachment| {
                build_attachment(self, attachment, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            })
            .collect();
        if let Some(depth_attachment) = builder.depth_attachment.as_ref() {
            attachments.push(build_attachment(
                self,
                depth_attachment,
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ));
        }

        let color_attachments: Vec<_> = builder
            .color_attachments
            .iter()
            .map(|attachment| {
                vk::AttachmentReference {
                    attachment: attachment.index,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                }
            })
            .collect();
        let depth_attachment = builder.depth_attachment.as_ref().map(|attachment| {
            vk::AttachmentReference {
                attachment: attachment.index,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            }
        });
        let mut dst_access_mask = vk::AccessFlags::empty();
        if builder.color_attachments.len() > 0 {
            dst_access_mask |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }
        if builder.depth_attachment.is_some() {
            dst_access_mask |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }
        let dependency = vk::SubpassDependency {
            dependency_flags: Default::default(),
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: Default::default(),
            src_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        };
        let mut subpass = vk::SubpassDescription {
            color_attachment_count: color_attachments.len() as _,
            p_color_attachments: color_attachments.as_ptr(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            ..Default::default()
        };
        if let Some(ref depth_attachment) = depth_attachment {
            subpass.p_depth_stencil_attachment = depth_attachment;
        }
        let renderpass_create_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
            ..Default::default()
        };
        let render_pass = self
            .device
            .create_render_pass(&renderpass_create_info, None)
            .unwrap();
        // let framebuffer_attachments = builder
        //     .color_attachments
        //     .iter()
        //     .chain(builder.depth_attachment.as_ref())
        //     .sorted_by(|l, r| l.index.cmp(&r.index))
        //     .map(|attachment| self.images.get(attachment.image.handle).image_view)
        //     .collect_vec();
        // // TODO: Proper resolution
        // let frame_buffer_create_info = vk::FramebufferCreateInfo {
        //     render_pass,
        //     attachment_count: framebuffer_attachments.len() as u32,
        //     p_attachments: framebuffer_attachments.as_ptr(),
        //     width: self.surface_resolution.width,
        //     height: self.surface_resolution.height,
        //     layers: 1,
        //     ..Default::default()
        // };
        // let framebuffer = self
        //     .device
        //     .create_framebuffer(&frame_buffer_create_info, None)
        //     .unwrap();
        self.renderpasses.insert(RenderpassData { render_pass })
    }
}
// use super::{CommandBuffer, Vulkan};
// use ash::version::DeviceV1_0;
// use ash::vk;
// use buffer::{Buffer, BufferProperty};
// use image::{Image, RenderTargetInfo};
// use pipeline::PipelineState;
// use renderpass::{VertexInput, VertexInputData};
// use std::marker::PhantomData;
// use std::ptr;
// pub struct RenderData {
//     context: Context<Vulkan>,
// }
// impl RenderApi<Vulkan> for Render<Vulkan> {
//     fn new(context: &Context<Vulkan>) -> Self {
//         let data = RenderData {
//             context: context.clone(),
//         };
//         Render { data }
//     }
//     fn draw_indexed<P, Vertex, Index>(
//         &self,
//         frame_buffer: vk::Framebuffer,
//         renderpass: &Renderpass<P, Vulkan>,
//         pipeline: Pipeline<Vulkan>,
//         vertex_buffer: &Buffer<Vertex, impl BufferProperty, Vulkan>,
//         index_buffer: &Buffer<Index, impl BufferProperty, Vulkan>,
//     ) where
//         P: Pass,
//     {
//         let ctx = &self.data.context;
//         let viewports = [vk::Viewport {
//             x: 0.0,
//             y: 0.0,
//             width: ctx.surface_resolution.width as f32,
//             height: ctx.surface_resolution.height as f32,
//             min_depth: 0.0,
//             max_depth: 1.0,
//         }];
//         let scissors = [vk::Rect2D {
//             offset: vk::Offset2D { x: 0, y: 0 },
//             extent: ctx.surface_resolution.clone(),
//         }];
//         let clear_values = [
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
//         CommandBuffer::record(ctx, |draw_command_buffer| {
//             let device = &ctx.device;
//             unsafe {
//                 let render_pass_begin_info = vk::RenderPassBeginInfo {
//                     s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
//                     p_next: ptr::null(),
//                     render_pass: renderpass.impl_render_pass.data.render_pass,
//                     framebuffer: frame_buffer,
//                     render_area: vk::Rect2D {
//                         offset: vk::Offset2D { x: 0, y: 0 },
//                         extent: ctx.surface_resolution.clone(),
//                     },
//                     clear_value_count: clear_values.len() as u32,
//                     p_clear_values: clear_values.as_ptr(),
//                 };
//                 device.cmd_begin_render_pass(
//                     draw_command_buffer,
//                     &render_pass_begin_info,
//                     vk::SubpassContents::INLINE,
//                 );
//                 device.cmd_bind_pipeline(
//                     draw_command_buffer,
//                     vk::PipelineBindPoint::GRAPHICS,
//                     pipeline.data.pipeline,
//                 );
//                 device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
//                 device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
//                 device.cmd_bind_vertex_buffers(
//                     draw_command_buffer,
//                     0,
//                     &[vertex_buffer.impl_buffer.buffer.buffer],
//                     &[0],
//                 );
//                 device.cmd_bind_index_buffer(
//                     draw_command_buffer,
//                     index_buffer.impl_buffer.buffer.buffer,
//                     0,
//                     vk::IndexType::UINT32,
//                 );
//                 device.cmd_draw_indexed(
//                     draw_command_buffer,
//                     index_buffer.impl_buffer.buffer.len as _,
//                     1,
//                     0,
//                     0,
//                     1,
//                 );
//                 // Or draw without the index buffer
//                 // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
//                 device.cmd_end_render_pass(draw_command_buffer);
//             }
//         });
//     }
// }

// pub struct RenderpassData {
//     pub context: Context,
//     pub renderpass: vk::RenderPass,
// }

// impl RenderpassApi for RenderpassData {}

// impl CreateRenderpass for Context {
//     fn new(&self, vertex_input: &[VertexInputData]) -> Renderpass {
//         let context = self;
//         let renderpass_attachments = [
//             vk::AttachmentDescription {
//                 format: context.surface_format.format,
//                 flags: vk::AttachmentDescriptionFlags::empty(),
//                 samples: vk::SampleCountFlags::TYPE_1,
//                 load_op: vk::AttachmentLoadOp::CLEAR,
//                 store_op: vk::AttachmentStoreOp::STORE,
//                 stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
//                 stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
//                 initial_layout: vk::ImageLayout::UNDEFINED,
//                 final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
//             },
//             vk::AttachmentDescription {
//                 format: vk::Format::D16_UNORM,
//                 flags: vk::AttachmentDescriptionFlags::empty(),
//                 samples: vk::SampleCountFlags::TYPE_1,
//                 load_op: vk::AttachmentLoadOp::CLEAR,
//                 store_op: vk::AttachmentStoreOp::DONT_CARE,
//                 stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
//                 stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
//                 initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
//                 final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
//             },
//         ];
//         let color_attachment_ref = vk::AttachmentReference {
//             attachment: 0,
//             layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
//         };
//         let depth_attachment_ref = vk::AttachmentReference {
//             attachment: 1,
//             layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
//         };
//         let dependency = vk::SubpassDependency {
//             dependency_flags: Default::default(),
//             src_subpass: vk::SUBPASS_EXTERNAL,
//             dst_subpass: Default::default(),
//             src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
//             src_access_mask: Default::default(),
//             dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
//                 | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
//             dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
//         };
//         let subpass = vk::SubpassDescription {
//             color_attachment_count: 1,
//             p_color_attachments: &color_attachment_ref,
//             p_depth_stencil_attachment: &depth_attachment_ref,
//             flags: Default::default(),
//             pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
//             input_attachment_count: 0,
//             p_input_attachments: ptr::null(),
//             p_resolve_attachments: ptr::null(),
//             preserve_attachment_count: 0,
//             p_preserve_attachments: ptr::null(),
//         };
//         let renderpass_create_info = vk::RenderPassCreateInfo {
//             s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
//             flags: Default::default(),
//             p_next: ptr::null(),
//             attachment_count: renderpass_attachments.len() as u32,
//             p_attachments: renderpass_attachments.as_ptr(),
//             subpass_count: 1,
//             p_subpasses: &subpass,
//             dependency_count: 1,
//             p_dependencies: &dependency,
//         };
//         let renderpass = unsafe {
//             context
//                 .device
//                 .create_render_pass(&renderpass_create_info, None)
//                 .unwrap()
//         };

//         let render_pass_data = RenderpassData {
//             context: context.clone(),
//             renderpass,
//         };
//         Renderpass {
//             renderpass: Box::new(render_pass_data),
//         }
//     }
// }
