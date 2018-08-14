use super::image::ImageData;
use super::Vulkan;
use ash::version::DeviceV1_0;
use ash::vk;
use context::Context;
use image::{Image};
use std::ptr;
use swapchain::{Swapchain, SwapchainApi};

pub struct SwapchainData {
    pub context: Context<Vulkan>,
    pub present_images: Vec<Image<Vulkan>>,
    pub swapchain: vk::SwapchainKHR,
}

impl SwapchainApi for Swapchain<Vulkan> {
    type Backend = Vulkan;
    fn new(ctx: &Context<Vulkan>) -> Swapchain<Vulkan> {
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
                }).nth(0)
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
                std::u32::MAX => ctx.surface_resolution,
                _ => surface_capabilities.current_extent,
            };
            let pre_transform = if surface_capabilities
                .supported_transforms
                .subset(vk::SurfaceTransformFlagsKHR::IDENTITY)
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
            let swapchain_loader = ash::extensions::Swapchain::new(&ctx.instance, &ctx.device)
                .expect("Unable to load swapchain");
            let swapchain_create_info = vk::SwapchainCreateInfoKHR {
                s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
                p_next: ptr::null(),
                flags: Default::default(),
                surface: ctx.surface,
                min_image_count: desired_image_count,
                image_color_space: surface_format.color_space,
                image_format: surface_format.format,
                image_extent: surface_resolution.clone(),
                image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                image_sharing_mode: vk::SharingMode::EXCLUSIVE,
                pre_transform: pre_transform,
                composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                present_mode: present_mode,
                clipped: 1,
                old_swapchain: vk::SwapchainKHR::null(),
                image_array_layers: 1,
                p_queue_family_indices: ptr::null(),
                queue_family_index_count: 0,
            };
            let swapchain = swapchain_loader
                .create_swapchain_khr(&swapchain_create_info, None)
                .unwrap();
            let present_images = swapchain_loader
                .get_swapchain_images_khr(swapchain)
                .unwrap();

            let present_images: Vec<Image<Vulkan>> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo {
                        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                        p_next: ptr::null(),
                        flags: Default::default(),
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: surface_format.format,
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
                    let data = ImageData {
                        context: ctx.clone(),
                        image,
                        image_view,
                    };
                    Image { data }
                }).collect();
            let data = SwapchainData {
                context: ctx.clone(),
                swapchain,
                present_images,
            };
            Swapchain { data }
        }
    }

    fn present_images(&self) -> &[Image<Vulkan>] {
        &self.data.present_images
    }
}
