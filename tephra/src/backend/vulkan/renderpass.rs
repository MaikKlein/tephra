use super::Vulkan;
use ash::version::DeviceV1_0;
use ash::vk;
use context::Context;
use renderpass::{ImplRenderpass, Pass, RenderpassApi};
use std::marker::PhantomData;
use std::ptr;
pub struct RenderpassData {
    pub context: Context<Vulkan>,
    pub render_pass: vk::RenderPass,
}

impl RenderpassApi<Vulkan> for ImplRenderpass<Vulkan> where {
    fn new(context: &Context<Vulkan>) -> Self {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: context.surface_format.format,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
        ];
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependency = vk::SubpassDependency {
            dependency_flags: Default::default(),
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: Default::default(),
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: Default::default(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        };
        let subpass = vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
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
        let render_pass = unsafe {
            context
                .device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };
        let render_pass_data = RenderpassData {
            context: context.clone(),
            render_pass,
        };
        ImplRenderpass {
            data: render_pass_data,
            _m: PhantomData,
        }
    }
}
