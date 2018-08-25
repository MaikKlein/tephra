use super::buffer;
use super::Context;
use super::{CommandBuffer, Vulkan};
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use buffer::Buffer;
use downcast::Downcast;
use image::{
    CreateImage, Image, ImageApi, ImageDesc, ImageLayout, RenderTarget, RenderTargetInfo,
    Resolution,
};
//use renderpass::{Pass, Renderpass};
use std::ptr;
// pub struct FramebufferData {}
pub struct ImageData {
    pub context: Context,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub desc: ImageDesc,
}

impl ImageApi for ImageData {
    fn desc(&self) -> &ImageDesc {
        &self.desc
    }
    fn copy_image(&self, target: &Image) {
        let target = target.downcast::<Vulkan>();
        let self_layout = get_image_layout(&self.desc);
        let target_layout = get_image_layout(&target.desc);
        let aspect_mask = get_aspect_mask(&target.desc);
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
                width: target.desc.resolution.width,
                height: target.desc.resolution.height,
                depth: 1,
            },
        };
        let command_buffer = CommandBuffer::record(&self.context, |command_buffer| unsafe {
            self.context.device.cmd_copy_image(
                command_buffer,
                self.image,
                vk::ImageLayout::UNDEFINED,
                //vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                target.image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                //vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[image_copy],
            );
        });
        self.context.present_queue.submit(
            &self.context,
            &[vk::PipelineStageFlags::TRANSFER],
            &[],
            &[],
            command_buffer,
        );
    }
}

fn get_image_layout(desc: &ImageDesc) -> vk::ImageLayout {
    match desc.layout {
        ImageLayout::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ImageLayout::Depth => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    }
}
fn get_aspect_mask(desc: &ImageDesc) -> vk::ImageAspectFlags {
    match desc.layout {
        ImageLayout::Color => vk::ImageAspectFlags::COLOR,
        ImageLayout::Depth => vk::ImageAspectFlags::DEPTH,
    }
}
impl CreateImage for Context {
    fn from_buffer(&self, buffer: Buffer<u8>) -> Image {
        unimplemented!()
    }
    fn allocate(&self, desc: ImageDesc) -> Image {
        let aspect_mask = match desc.layout {
            ImageLayout::Color => vk::ImageAspectFlags::COLOR,
            ImageLayout::Depth => vk::ImageAspectFlags::DEPTH,
        };
        let format = match desc.layout {
            ImageLayout::Color => vk::Format::R8G8B8A8_UNORM,
            ImageLayout::Depth => vk::Format::D16_UNORM,
        };
        let usage = match desc.layout {
            ImageLayout::Color => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ImageLayout::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        };
        //let usage = usage | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;

        let access = match desc.layout {
            //ImageLayout::Color => vk::AccessFlags::empty(),
            ImageLayout::Color => {
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            }
            ImageLayout::Depth => {
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
        };
        let access = vk::AccessFlags::empty();
        let target_layout = match desc.layout {
            ImageLayout::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::Depth => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let ctx = self;
        unsafe {
            let device_memory_properties = ctx
                .instance
                .get_physical_device_memory_properties(ctx.pdevice);
            let depth_image_create_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                image_type: vk::ImageType::TYPE_2D,
                format,
                extent: vk::Extent3D {
                    width: desc.resolution.width,
                    height: desc.resolution.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
                initial_layout: vk::ImageLayout::UNDEFINED,
            };
            let depth_image = ctx
                .device
                .create_image(&depth_image_create_info, None)
                .unwrap();
            let depth_image_memory_req = ctx.device.get_image_memory_requirements(depth_image);
            let depth_image_memory_index =
                buffer::find_memorytype_index(
                    &depth_image_memory_req,
                    &device_memory_properties,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                ).expect("Unable to find suitable memory index for depth image.");

            let depth_image_allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                p_next: ptr::null(),
                allocation_size: depth_image_memory_req.size,
                memory_type_index: depth_image_memory_index,
            };
            let depth_image_memory = ctx
                .device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap();
            ctx.device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Unable to bind depth image memory");
            // let command_buffer = CommandBuffer::record(ctx, |command_buffer| {
            //     let layout_transition_barrier = vk::ImageMemoryBarrier {
            //         s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            //         p_next: ptr::null(),
            //         src_access_mask: Default::default(),
            //         dst_access_mask: access,
            //         old_layout: vk::ImageLayout::UNDEFINED,
            //         new_layout: target_layout,
            //         src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            //         dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            //         image: depth_image,
            //         subresource_range: vk::ImageSubresourceRange {
            //             aspect_mask,
            //             base_mip_level: 0,
            //             level_count: 1,
            //             base_array_layer: 0,
            //             layer_count: 1,
            //         },
            //     };
            //     ctx.device.cmd_pipeline_barrier(
            //         command_buffer,
            //         vk::PipelineStageFlags::TOP_OF_PIPE,
            //         vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            //         vk::DependencyFlags::empty(),
            //         &[],
            //         &[],
            //         &[layout_transition_barrier],
            //     );
            // });
            // ctx.present_queue.submit(ctx, &[], &[], &[], command_buffer);
            let depth_image_view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: depth_image_create_info.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image: depth_image,
            };
            let image_view = ctx
                .device
                .create_image_view(&depth_image_view_info, None)
                .unwrap();
            let data = ImageData {
                context: ctx.clone(),
                image_view,
                image: depth_image,
                desc,
            };
            Image {
                data: Box::new(data),
            }
        }
    }
}

// impl FramebufferApi for FramebufferData {
// }
// impl CreateFramebuffer for FramebufferData {
//     fn new(
//         &self,
//         render_target_info: &RenderTargetInfo,
//     ) -> Self {
//         // unsafe {
//         //     let render_target_info = target.render_target();
//         //     let framebuffer_attachments: Vec<vk::ImageView> = render_target_info
//         //         .image_views
//         //         .iter()
//         //         .map(|&image| {
//         //             let image_data = image.data.downcast_ref::<ImageData>().unwrap();
//         //             image_data.image_view
//         //         }).collect();
//         //     let frame_buffer_create_info = vk::FramebufferCreateInfo {
//         //         s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
//         //         p_next: ptr::null(),
//         //         flags: Default::default(),
//         //         render_pass: renderpass.impl_render_pass.data.render_pass,
//         //         attachment_count: framebuffer_attachments.len() as u32,
//         //         p_attachments: framebuffer_attachments.as_ptr(),
//         //         width: context.surface_resolution.width,
//         //         height: context.surface_resolution.height,
//         //         layers: 1,
//         //     };
//         //     context
//         //         .device
//         //         .create_framebuffer(&frame_buffer_create_info, None)
//         //         .unwrap();
//         // }
//         unimplemented!()
//     }
// }
