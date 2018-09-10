use ash::extensions::{DebugReport, DebugUtils, Surface, Swapchain, XlibSurface};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::{Device, Entry, Instance};
use backend::BackendApi;
use context;
use context::ContextApi;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, Drop};
use std::ptr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use thread_local_object::ThreadLocal;
use winit;
pub mod buffer;
pub mod commandbuffer;
pub mod descriptor;
pub mod image;
pub mod pipeline;
pub mod render;
pub mod renderpass;
pub mod shader;
pub mod swapchain;

#[derive(Copy, Clone)]
pub struct Vulkan;
impl BackendApi for Vulkan {
    type Shader = shader::ShaderData;
    type Context = Context;
    type Buffer = buffer::BufferData;
    type Image = image::ImageData;
    type Swapchain = swapchain::SwapchainData;
    type Render = render::Render;
    type Compute = render::Compute;
    type Descriptor = descriptor::Descriptor;
    type Layout = descriptor::Layout;
}

#[derive(Clone)]
pub struct ThreadLocalCommandPool {
    queue_family_index: vk::uint32_t,
    thread_local_command_pool: Arc<ThreadLocal<CommandPool>>,
}

impl ThreadLocalCommandPool {
    pub fn new(queue_family_index: vk::uint32_t) -> Self {
        ThreadLocalCommandPool {
            queue_family_index,
            thread_local_command_pool: Arc::new(ThreadLocal::new()),
        }
    }

    fn get_command_buffer(&self, context: &Context) -> RecordCommandBuffer {
        let has_local_value = self.thread_local_command_pool.get(|value| value.is_some());
        if !has_local_value {
            let _ = self
                .thread_local_command_pool
                .set(CommandPool::new(context, self.queue_family_index));
        }

        self.thread_local_command_pool.get_mut(|pool| {
            pool.expect("Should have local pool")
                .get_command_buffer(context)
        })
    }
}

pub struct CommandPool {
    pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    sender: Sender<vk::CommandBuffer>,
    receiver: Receiver<vk::CommandBuffer>,
}

impl CommandPool {
    fn new(context: &Context, queue_family_index: vk::uint32_t) -> Self {
        let pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index: queue_family_index,
        };
        let pool = unsafe {
            context
                .device
                .create_command_pool(&pool_create_info, None)
                .unwrap()
        };
        let (sender, receiver) = channel();

        CommandPool {
            pool,
            command_buffers: Vec::new(),
            sender,
            receiver,
        }
    }

    fn allocate_command_buffers(&mut self, context: &Context, count: u32) {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: self.pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: count,
        };
        unsafe {
            let v: Vec<_> = context
                .device
                .allocate_command_buffers(&alloc_info)
                .unwrap();
            self.command_buffers.extend(v.into_iter());
        }
    }
    pub fn get_command_buffer(&mut self, context: &Context) -> RecordCommandBuffer {
        {
            let reset_command_buffer_iter = self.receiver.try_iter().map(|command_buffer| {
                unsafe {
                    context
                        .device
                        .reset_command_buffer(
                            command_buffer,
                            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                        )
                        .expect("Reset command buffer failed.");
                }
                command_buffer
            });
            // Add queued command buffers
            self.command_buffers.extend(reset_command_buffer_iter);
        }
        let free_command_buffer = self.command_buffers.pop().unwrap_or_else(|| {
            // If no buffer is available, we need to allocate
            self.allocate_command_buffers(context, 10);
            self.command_buffers.pop().expect("CommandBuffer")
        });

        RecordCommandBuffer {
            inner: free_command_buffer,
            sender: self.sender.clone(),
            _m: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Queue {
    pub inner: Mutex<vk::Queue>,
}

impl Queue {
    pub fn new(queue: vk::Queue) -> Queue {
        Queue {
            inner: Mutex::new(queue),
        }
    }

    pub fn submit(
        &self,
        context: &Context,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        command_buffer: CommandBuffer,
    ) {
        unsafe {
            let fence_create_info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FenceCreateFlags::empty(),
            };
            let submit_fence = context
                .device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.");
            let queue = self.inner.lock();
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_mask.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: &command_buffer.inner,
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
            };
            context
                .device
                .queue_submit(*queue, &[submit_info], submit_fence)
                .expect("Unable to submit");
            // TODO: Future
            context
                .device
                .wait_for_fences(&[submit_fence], true, u64::max_value())
                .expect("Unable to wait");
        }
    }
}

#[derive(Debug)]
pub struct RecordCommandBuffer {
    inner: vk::CommandBuffer,
    sender: Sender<vk::CommandBuffer>,
    _m: PhantomData<*const ()>,
}

#[derive(Debug)]
pub struct CommandBuffer {
    inner: vk::CommandBuffer,
    sender: Sender<vk::CommandBuffer>,
}

impl CommandBuffer {
    pub fn record<F>(context: &Context, name: &str, mut f: F) -> Self
    where
        F: FnMut(vk::CommandBuffer),
    {
        let RecordCommandBuffer {
            inner: command_buffer,
            sender,
            ..
        } = context.command_pool.get_command_buffer(context);
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };
        use ash::vk::Handle;
        let cname = CString::new(name).unwrap();

        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type: vk::ObjectType::COMMAND_BUFFER,
            object_handle: command_buffer.as_raw(),
            p_object_name: cname.as_ptr(),
            ..Default::default()
        };
        unsafe {
            context
                .debug_utils_loader
                .debug_utils_set_object_name_ext(context.device.handle(), &name_info)
                .expect("util name");
            context
                .device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
            f(command_buffer);
            context
                .device
                .end_command_buffer(command_buffer)
                .expect("End commandbuffer");
            CommandBuffer {
                inner: command_buffer,
                sender,
            }
        }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        // Reclaim the command buffer by sending it to the correct pool
        self.sender.send(self.inner).expect("unable to send");
    }
}

#[derive(Clone)]
pub struct Context {
    inner: Arc<InnerContext>,
}

impl Deref for Context {
    type Target = InnerContext;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
pub struct InnerContext {
    pub entry: Entry<V1_0>,
    pub instance: Instance<V1_0>,
    pub device: Device<V1_0>,
    pub window: winit::Window,
    pub events_loop: RefCell<winit::EventsLoop>,
    pub physical_device: vk::PhysicalDevice,
    pub command_pool: ThreadLocalCommandPool,
    //pub graphics_queue: Mutex<vk::Queue>,
    //command_pool: CommandPool,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_utils_loader: DebugUtils,
    pub debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    //pub debug_report_loader: DebugReport,
    //pub window: winit::Window,
    //pub events_loop: RefCell<winit::EventsLoop>,
    //pub debug_call_back: vk::DebugReportCallbackEXT,
    pub pdevice: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    // pub swapchain: vk::SwapchainKHR,
    // pub present_images: Vec<vk::Image>,
    // pub present_image_views: Vec<vk::ImageView>,
    pub pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,
    pub pipeline_cache: vk::PipelineCache,
}
impl ContextApi for Context {}
impl Context {
    // pub fn render_loop<F: FnMut()>(&self, mut f: F) {
    //     use winit::*;
    //     self.events_loop.borrow_mut().run_forever(|event| {
    //         f();
    //         match event {
    //             Event::WindowEvent { event, .. } => match event {
    //                 WindowEvent::KeyboardInput { input, .. } => {
    //                     if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
    //                         ControlFlow::Break
    //                     } else {
    //                         ControlFlow::Continue
    //                     }
    //                 }
    //                 WindowEvent::Closed => winit::ControlFlow::Break,
    //                 _ => ControlFlow::Continue,
    //             },
    //             _ => ControlFlow::Continue,
    //         }
    //     });
    // }
    pub fn new() -> context::Context {
        unsafe {
            let window_width = 1000;
            let window_height = 1000;
            let events_loop = winit::EventsLoop::new();
            let window = winit::WindowBuilder::new()
                .with_title("Ash - Example")
                .with_dimensions((window_width, window_height).into())
                .build(&events_loop)
                .unwrap();
            let entry = Entry::new().unwrap();
            let app_name = CString::new("VulkanTriangle").unwrap();
            let raw_name = app_name.as_ptr();

            let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();
            let extension_names_raw = extension_names();
            let appinfo = vk::ApplicationInfo {
                p_application_name: raw_name,
                s_type: vk::StructureType::APPLICATION_INFO,
                p_next: ptr::null(),
                application_version: 0,
                p_engine_name: raw_name,
                engine_version: 0,
                api_version: vk_make_version!(1, 0, 36),
            };
            let create_info = vk::InstanceCreateInfo {
                s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                p_application_info: &appinfo,
                pp_enabled_layer_names: layers_names_raw.as_ptr(),
                enabled_layer_count: layers_names_raw.len() as u32,
                pp_enabled_extension_names: extension_names_raw.as_ptr(),
                enabled_extension_count: extension_names_raw.len() as u32,
            };
            let instance: Instance<V1_0> = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");
            // let debug_info = vk::DebugReportCallbackCreateInfoEXT {
            //     s_type: vk::StructureType::DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
            //     p_next: ptr::null(),
            //     flags: vk::DebugReportFlagsEXT::ERROR
            //         | vk::DebugReportFlagsEXT::WARNING
            //         | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
            //     pfn_callback: vulkan_debug_callback,
            //     p_user_data: ptr::null_mut(),
            // };
            // let debug_report_loader =
            //     DebugReport::new(&entry, &instance).expect("Unable to load debug report");
            let debug_utils_loader = DebugUtils::new(&entry, &instance).expect("utils");
            let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
                s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                p_next: ::std::ptr::null(),
                flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
                p_user_data: ::std::ptr::null_mut(),
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                pfn_user_callback: debug_utils_callback,
            };

            let debug_utils_messenger = debug_utils_loader
                .create_debug_utils_messenger_ext(&messenger_create_info, None)
                .expect("messenger");
            // let debug_call_back = debug_report_loader
            //     .create_debug_report_callback_ext(&debug_info, None)
            //     .unwrap();
            let surface = create_surface(&entry, &instance, &window).unwrap();
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Physical device error");
            let surface_loader =
                Surface::new(&entry, &instance).expect("Unable to load the Surface extension");
            let (pdevice, queue_family_index) = pdevices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.subset(vk::QueueFlags::GRAPHICS)
                                    && surface_loader.get_physical_device_surface_support_khr(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    );
                            match supports_graphic_and_surface {
                                true => Some((*pdevice, index)),
                                _ => None,
                            }
                        })
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Couldn't find suitable device.");
            println!("{:#?}", entry.enumerate_instance_extension_properties());
            println!(
                "{:#?}",
                instance.enumerate_device_extension_properties(pdevice)
            );
            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                queue_family_index: queue_family_index as u32,
                p_queue_priorities: priorities.as_ptr(),
                queue_count: priorities.len() as u32,
            };
            let device_create_info = vk::DeviceCreateInfo {
                s_type: vk::StructureType::DEVICE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                queue_create_info_count: 1,
                p_queue_create_infos: &queue_info,
                enabled_layer_count: 0,
                pp_enabled_layer_names: ptr::null(),
                enabled_extension_count: device_extension_names_raw.len() as u32,
                pp_enabled_extension_names: device_extension_names_raw.as_ptr(),
                p_enabled_features: &features,
            };
            let device: Device<V1_0> = instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap();
            let present_queue = device.get_device_queue(queue_family_index as u32, 0);
            let present_queue = Queue::new(present_queue);

            let surface_formats = surface_loader
                .get_physical_device_surface_formats_khr(pdevice, surface)
                .unwrap();
            let surface_format = surface_formats
                .iter()
                .map(|sfmt| match sfmt.format {
                    vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
                        format: vk::Format::B8G8R8_UNORM,
                        color_space: sfmt.color_space,
                    },
                    _ => *sfmt,
                })
                .nth(0)
                .expect("Unable to find suitable surface format.");
            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities_khr(pdevice, surface)
                .unwrap();
            let surface_resolution = match surface_capabilities.current_extent.width {
                ::std::u32::MAX => vk::Extent2D {
                    width: window_width,
                    height: window_height,
                },
                _ => surface_capabilities.current_extent,
            };
            let swapchain_loader =
                Swapchain::new(&instance, &device).expect("Unable to load swapchain");
            // let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            //     s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            //     p_next: ptr::null(),
            //     flags: Default::default(),
            //     surface: surface,
            //     min_image_count: desired_image_count,
            //     image_color_space: surface_format.color_space,
            //     image_format: surface_format.format,
            //     image_extent: surface_resolution.clone(),
            //     image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            //     image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            //     pre_transform: pre_transform,
            //     composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            //     present_mode: present_mode,
            //     clipped: 1,
            //     old_swapchain: vk::SwapchainKHR::null(),
            //     image_array_layers: 1,
            //     p_queue_family_indices: ptr::null(),
            //     queue_family_index_count: 0,
            // };
            // let swapchain = swapchain_loader
            //     .create_swapchain_khr(&swapchain_create_info, None)
            //     .unwrap();
            let pool_create_info = vk::CommandPoolCreateInfo {
                s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                queue_family_index: queue_family_index,
            };
            let pool = device.create_command_pool(&pool_create_info, None).unwrap();
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: ptr::null(),
                command_buffer_count: 2,
                command_pool: pool,
                level: vk::CommandBufferLevel::PRIMARY,
            };
            let command_buffers = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();
            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

            // let present_images = swapchain_loader
            //     .get_swapchain_images_khr(swapchain)
            //     .unwrap();
            // let present_image_views: Vec<vk::ImageView> = present_images
            //     .iter()
            //     .map(|&image| {
            //         let create_view_info = vk::ImageViewCreateInfo {
            //             s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            //             p_next: ptr::null(),
            //             flags: Default::default(),
            //             view_type: vk::ImageViewType::TYPE_2D,
            //             format: surface_format.format,
            //             components: vk::ComponentMapping {
            //                 r: vk::ComponentSwizzle::R,
            //                 g: vk::ComponentSwizzle::G,
            //                 b: vk::ComponentSwizzle::B,
            //                 a: vk::ComponentSwizzle::A,
            //             },
            //             subresource_range: vk::ImageSubresourceRange {
            //                 aspect_mask: vk::ImageAspectFlags::COLOR,
            //                 base_mip_level: 0,
            //                 level_count: 1,
            //                 base_array_layer: 0,
            //                 layer_count: 1,
            //             },
            //             image: image,
            //         };
            //         device.create_image_view(&create_view_info, None).unwrap()
            //     }).collect();
            let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);
            let depth_image_create_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::D16_UNORM,
                extent: vk::Extent3D {
                    width: surface_resolution.width,
                    height: surface_resolution.height,
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
            let depth_image = device.create_image(&depth_image_create_info, None).unwrap();
            let depth_image_memory_req = device.get_image_memory_requirements(depth_image);
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
            let depth_image_memory = device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap();
            device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Unable to bind depth image memory");
            record_submit_commandbuffer(
                &device,
                setup_command_buffer,
                &present_queue.inner,
                &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
                &[],
                &[],
                |device, setup_command_buffer| {
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
                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barrier],
                    );
                },
            );
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
            let depth_image_view = device
                .create_image_view(&depth_image_view_info, None)
                .unwrap();
            let semaphore_create_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
            };
            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::default();
            let pipeline_cache = device
                .create_pipeline_cache(&pipeline_cache_create_info, None)
                .expect("pipeline cache");
            let context = InnerContext {
                command_pool: ThreadLocalCommandPool::new(queue_family_index),
                entry,
                physical_device: pdevice,
                events_loop: RefCell::new(events_loop),
                window,
                instance: instance,
                device: device,
                queue_family_index: queue_family_index,
                pdevice: pdevice,
                device_memory_properties: device_memory_properties,
                //window: window,
                surface_loader: surface_loader,
                surface_format: surface_format,
                present_queue: present_queue,
                surface_resolution: surface_resolution,
                swapchain_loader: swapchain_loader,
                // present_images: present_images,
                // present_image_views: present_image_views,
                pool: pool,
                draw_command_buffer: draw_command_buffer,
                setup_command_buffer: setup_command_buffer,
                depth_image: depth_image,
                depth_image_view: depth_image_view,
                present_complete_semaphore: present_complete_semaphore,
                rendering_complete_semaphore: rendering_complete_semaphore,
                surface: surface,
                // debug_call_back: debug_call_back,
                // debug_report_loader: debug_report_loader,
                depth_image_memory: depth_image_memory,
                pipeline_cache,
                debug_utils_loader,
                debug_utils_messenger,
            };
            let context = Context {
                inner: Arc::new(context),
            };
            context::Context {
                // FIXME: Only one Arc
                context: Arc::new(context),
            }
        }
    }
}

impl Drop for InnerContext {
    fn drop(&mut self) {
        // unsafe {
        //     // self.device.destroy_device(None);
        //     // self.instance.destroy_instance(None);
        // }
    }
}
// impl Context {
//     pub fn new() -> context::Context<Vulkan> {

//     }
// }
#[cfg(all(unix, not(target_os = "android")))]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader =
        XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
    xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
}

#[cfg(windows)]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::GetWindow;
    use winit::os::windows::WindowExt;

    let hwnd = window.get_hwnd() as HWND;
    let hinstance = GetWindow(hwnd, 0) as *const vk::c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance: hinstance,
        hwnd: hwnd as *const vk::c_void,
    };
    let win32_surface_loader =
        Win32Surface::new(entry, instance).expect("Unable to load win32 surface");
    win32_surface_loader.create_win32_surface_khr(&win32_create_info, None)
}

#[cfg(all(unix, not(target_os = "android")))]
fn extension_names() -> Vec<*const i8> {
    vec![
        DebugUtils::name().as_ptr(),
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

pub fn record_submit_commandbuffer<D: DeviceV1_0, F: FnOnce(&D, vk::CommandBuffer)>(
    device: &D,
    command_buffer: vk::CommandBuffer,
    submit_queue: &Mutex<vk::Queue>,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        let submit_queue = *submit_queue.lock();
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };
        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin commandbuffer");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("End commandbuffer");
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::empty(),
        };
        let submit_fence = device
            .create_fence(&fence_create_info, None)
            .expect("Create fence failed.");
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_mask.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        };
        device
            .queue_submit(submit_queue, &[submit_info], submit_fence)
            .expect("queue submit failed.");
        device
            .wait_for_fences(&[submit_fence], true, ::std::u64::MAX)
            .expect("Wait for fence failed.");
        device.destroy_fence(submit_fence, None);
    }
}
unsafe extern "system" fn debug_utils_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut vk::c_void,
) -> vk::Bool32 {
    if !p_callback_data.is_null() {
        let data = &*p_callback_data;
        for i in 0..data.object_count {
            let obj = data.p_objects.offset(i as isize).read();
            println!(
                "Object: [{}] {} 0x{:x}",
                obj.object_type,
                CStr::from_ptr(obj.p_object_name).to_str().unwrap(),
                obj.object_handle
            );
        }
        println!("Message ID: {:?}", CStr::from_ptr(data.p_message_id_name));
        println!("Message: {:?}", CStr::from_ptr(data.p_message));
        println!("");
    }
    0
}
// unsafe extern "system" fn vulkan_debug_callback(
//     _: vk::DebugReportFlagsEXT,
//     _: vk::DebugReportObjectTypeEXT,
//     _: vk::uint64_t,
//     _: vk::size_t,
//     _: vk::int32_t,
//     _: *const vk::c_char,
//     p_message: *const vk::c_char,
//     _: *mut vk::c_void,
// ) -> u32 {
//     println!("{:?}", CStr::from_ptr(p_message));
//     1
// }
