use super::image::{into_format, ImageData};
use super::CommandBuffer;
use super::Context;
use ash::extensions;
use ash::version::DeviceV1_0;
use ash::vk;
use std::ops::Drop;
use std::ptr;
use tephra::{
    image::{Format, Image, ImageApi, ImageDesc, ImageLayout, Resolution},
    swapchain::{CreateSwapchain, Swapchain, SwapchainApi, SwapchainError},
};

pub struct SwapchainData {
    pub context: Context,
    pub present_images: Vec<Image>,
    pub swapchain: vk::SwapchainKHR,
    pub resolution: Resolution,
}

impl Drop for SwapchainData {
    fn drop(&mut self) {
        unsafe {
            self.context
                .swapchain_loader
                .destroy_swapchain_khr(self.swapchain, None);
        }
    }
}

impl SwapchainApi for SwapchainData {
    fn format(&self) -> Format {
        into_format(self.context.surface_format.format)
    }
    fn copy_and_present(&self, image: Image) {
        let index = self.aquire_next_image().expect("acquire");
        let present_image = &self.present_images()[index as usize];
        self.context.copy_image(image.handle, present_image.handle);
        self.present(index);
    }
    fn recreate(&mut self) {
        let new_swapchain = create_swapchain(&self.context, Some(self.swapchain));
        *self = new_swapchain;
    }
    fn resolution(&self) -> Resolution {
        self.resolution
    }
    fn present_images(&self) -> &[Image] {
        &self.present_images
    }
    fn aquire_next_image(&self) -> Result<u32, SwapchainError> {
        unsafe {
            self.context
                .swapchain_loader
                .acquire_next_image_khr(
                    self.swapchain,
                    ::std::u64::MAX,
                    self.context.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .map(|e| e.0)
                .map_err(|err| match err {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => SwapchainError::OutOfDate,
                    vk::Result::SUBOPTIMAL_KHR => SwapchainError::Suboptimal,
                    err => {
                        println!("{:?}", err);
                        println!("{:?}", vk::Result::ERROR_OUT_OF_DATE_KHR);
                        SwapchainError::Unknown
                    }
                })
        }
    }
    fn present(&self, index: u32) {
        unsafe {
            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: &self.context.present_complete_semaphore,
                swapchain_count: 1,
                p_swapchains: &self.swapchain,
                p_image_indices: &index,
                p_results: ptr::null_mut(),
            };
            self.context
                .swapchain_loader
                .queue_present_khr(*self.context.present_queue.inner.lock(), &present_info)
                .unwrap();
        }
    }
}

unsafe fn get_swapchain_images(
    ctx: &Context,
    swapchain: vk::SwapchainKHR,
    resolution: Resolution,
) -> Vec<Image> {
    let present_images = ctx
        .swapchain_loader
        .get_swapchain_images_khr(swapchain)
        .unwrap();
    present_images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: ctx.surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image: image,
            };
            let image_view = ctx
                .device
                .create_image_view(&create_view_info, None)
                .unwrap();
            let desc = ImageDesc {
                resolution,
                layout: ImageLayout::Color,
                format: into_format(ctx.surface_format.format),
            };
            let data = ImageData {
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                image,
                image_view,
                desc,
            };
            let handle = ctx.images.insert(data);
            Image { handle }
        })
        .collect()
}
fn create_swapchain(ctx: &Context, old_swapchain: Option<vk::SwapchainKHR>) -> SwapchainData {
    unsafe {
        let surface_formats = ctx
            .surface_loader
            .get_physical_device_surface_formats_khr(ctx.pdevice, ctx.surface)
            .unwrap();
        let surface_format = surface_formats
            .iter()
            .map(|sfmt| match sfmt.format {
                vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
                    format: vk::Format::B8G8R8_UNORM,
                    color_space: sfmt.color_space,
                },
                _ => sfmt.clone(),
            })
            .nth(0)
            .expect("Unable to find suitable surface format.");
        let surface_capabilities = ctx
            .surface_loader
            .get_physical_device_surface_capabilities_khr(ctx.pdevice, ctx.surface)
            .unwrap();
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let surface_resolution = match surface_capabilities.current_extent.width {
            ::std::u32::MAX => ctx.surface_resolution,
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = ctx
            .surface_loader
            .get_physical_device_surface_present_modes_khr(ctx.pdevice, ctx.surface)
            .unwrap();
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = extensions::Swapchain::new(&ctx.instance, &ctx.device);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: ctx.surface,
            min_image_count: desired_image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: surface_resolution.clone(),
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: pre_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: 1,
            old_swapchain: old_swapchain.unwrap_or(vk::SwapchainKHR::null()),
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };
        let swapchain = swapchain_loader
            .create_swapchain_khr(&swapchain_create_info, None)
            .unwrap();

        let resolution = Resolution {
            width: surface_resolution.width,
            height: surface_resolution.height,
        };
        let present_images = get_swapchain_images(ctx, swapchain, resolution);
        for &image in &present_images {
            let vkimage = ctx.images.get(image.handle);
            let command_buffer = CommandBuffer::record(ctx, "SwapchainBarrier", |command_buffer| {
                let present_barrier = vk::ImageMemoryBarrier {
                    s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
                    p_next: ptr::null(),
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::empty(),
                    old_layout: vk::ImageLayout::UNDEFINED,
                    new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: vkimage.image,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                ctx.device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[present_barrier],
                );
            });
            ctx.present_queue.submit(ctx, &[], &[], &[], command_buffer);
        }
        // for image in &present_images {
        //     let barrier = vk::ImageMemoryBarrier {
        //         s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        //         p_next: ptr::null(),

        //     };
        // }
        SwapchainData {
            context: ctx.clone(),
            swapchain,
            present_images,
            resolution,
        }
    }
}

impl CreateSwapchain for Context {
    fn new(&self) -> Swapchain {
        let data = create_swapchain(self, None);
        Swapchain {
            data: Box::new(data),
        }
    }
}
