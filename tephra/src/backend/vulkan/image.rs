use super::buffer;
use super::{CommandBuffer, Vulkan};
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use context::Context;
use image::{Image, ImageApi, Resolution};
use std::ptr;
pub struct FramebufferData {}
pub struct ImageData {
    pub context: Context<Vulkan>,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
}

impl ImageApi for Image<Vulkan> {
    type Backend = Vulkan;
    fn create_depth(ctx: &Context<Vulkan>, resolution: Resolution) -> Image<Vulkan> {
        Self::allocate(ctx, resolution)
    }
    fn allocate(ctx: &Context<Vulkan>, resolution: Resolution) -> Image<Vulkan> {
        unsafe {
            let device_memory_properties = ctx
                .instance
                .get_physical_device_memory_properties(ctx.pdevice);
            let depth_image_create_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::D16_UNORM,
                extent: vk::Extent3D {
                    width: resolution.width,
                    height: resolution.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
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
            let depth_image_memory_index = buffer::find_memorytype_index(
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
            let command_buffer = CommandBuffer::record(ctx, |command_buffer| {
                let layout_transition_barrier = vk::ImageMemoryBarrier {
                    s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                    p_next: ptr::null(),
                    src_access_mask: Default::default(),
                    dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: depth_image,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barrier],
                );
            });
            ctx.present_queue.submit(ctx, &[], &[], &[], command_buffer);
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
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
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
            };
            Image { data }
        }
    }
}
