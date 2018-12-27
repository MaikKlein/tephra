use super::buffer;
use super::CommandBuffer;
use super::Context;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::ptr;
use tephra::{
    buffer::Buffer,
    image::{Format, ImageApi, ImageDesc, ImageHandle, ImageLayout},
};
pub(crate) fn into_format(vk_format: vk::Format) -> Format {
    Format::from_raw(vk_format.as_raw())
}
pub(crate) fn from_format(format: Format) -> vk::Format {
    vk::Format::from_raw(format.as_raw())
}
pub struct ImageData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub layout: vk::ImageLayout,
    pub desc: ImageDesc,
}

impl ImageApi for Context {
    fn allocate_image(&self, desc: ImageDesc) -> ImageHandle {
        let aspect_mask = match desc.layout {
            ImageLayout::Color => vk::ImageAspectFlags::COLOR,
            ImageLayout::Depth => vk::ImageAspectFlags::DEPTH,
        };
        let format = from_format(desc.format);
        let usage = match desc.layout {
            ImageLayout::Color => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ImageLayout::Depth => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        };
        let usage = usage | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;

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
            let depth_image_memory_index = buffer::find_memorytype_index(
                &depth_image_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Unable to find suitable memory index for depth image.");

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
            let command_buffer = CommandBuffer::record(ctx, "ImageAllocate", |command_buffer| {
                let layout_transition_barrier = vk::ImageMemoryBarrier {
                    s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                    p_next: ptr::null(),
                    src_access_mask: Default::default(),
                    dst_access_mask: access,
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: target_layout,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: depth_image,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
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
                layout: get_image_layout(&desc),
                image_view,
                image: depth_image,
                desc,
            };
            self.images.insert(data)
        }
    }
    fn from_buffer(&self, _buffer: Buffer<u8>) -> ImageHandle {
        unimplemented!()
    }
    fn desc(&self, handle: ImageHandle) -> ImageDesc {
        let data = self.images.get(handle);
        data.desc.clone()
    }
    fn copy_image(&self, src: ImageHandle, dst: ImageHandle) {
        let src_data = self.images.get(src);
        let dst_data = self.images.get(dst);
        let _self_layout = get_image_layout(&src_data.desc);
        let _target_layout = get_image_layout(&dst_data.desc);
        let aspect_mask = get_aspect_mask(&dst_data.desc);
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
                width: dst_data.desc.resolution.width,
                height: dst_data.desc.resolution.height,
                depth: 1,
            },
        };
        let command_buffer = CommandBuffer::record(self, "DST", |command_buffer| {
            let layout_transition_barrier = vk::ImageMemoryBarrier {
                s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                p_next: ptr::null(),
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::empty(),
                old_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: dst_data.image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };
            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barrier],
                );
            }
        });
        self.present_queue
            .submit(self, &[], &[], &[], command_buffer);
        let command_buffer = CommandBuffer::record(self, "ToSrcOptimal", |command_buffer| {
            let layout_transition_barrier = vk::ImageMemoryBarrier {
                s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                p_next: ptr::null(),
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::empty(),
                old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: src_data.image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };
            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barrier],
                );
            }
        });
        self.present_queue
            .submit(self, &[], &[], &[], command_buffer);
        let command_buffer = CommandBuffer::record(self, "ImageCopy", |command_buffer| unsafe {
            self.device.cmd_copy_image(
                command_buffer,
                src_data.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst_data.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[image_copy],
            );
        });
        self.present_queue.submit(
            self,
            &[vk::PipelineStageFlags::TRANSFER],
            &[],
            &[],
            command_buffer,
        );
        let command_buffer = CommandBuffer::record(self, "FromSrc", |command_buffer| {
            let layout_transition_barrier = vk::ImageMemoryBarrier {
                s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                p_next: ptr::null(),
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: src_data.image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
            };
            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barrier],
                );
            }
        });
        self.present_queue
            .submit(self, &[], &[], &[], command_buffer);
        let command_buffer =
            CommandBuffer::record(self, "ToPresentFromImageCopy", |command_buffer| {
                let layout_transition_barrier = vk::ImageMemoryBarrier {
                    s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                    p_next: ptr::null(),
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::empty(),
                    old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: dst_data.image,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                unsafe {
                    self.device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barrier],
                    );
                }
            });
        self.present_queue
            .submit(self, &[], &[], &[], command_buffer);
    }
}

fn get_image_layout(desc: &ImageDesc) -> vk::ImageLayout {
    match desc.layout {
        ImageLayout::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ImageLayout::Depth => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    }
}
pub(crate) fn get_aspect_mask(desc: &ImageDesc) -> vk::ImageAspectFlags {
    match desc.layout {
        ImageLayout::Color => vk::ImageAspectFlags::COLOR,
        ImageLayout::Depth => vk::ImageAspectFlags::DEPTH,
    }
}
