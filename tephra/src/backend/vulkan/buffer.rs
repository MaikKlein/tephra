use super::Context;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use buffer::{BufferApi, BufferError, BufferUsage, CreateBuffer, MappingError, Property};
use std::ptr;

/// Vulkan specifc data
pub struct BufferData {
    pub context: Context,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,
}

impl Drop for BufferData {
    fn drop(&mut self) {
        // unsafe {
        //     // self.context.device.destroy_buffer(self.buffer, None);
        //     // self.context.device.free_memory(self.memory, None);
        // }
    }
}

fn bitflag_to_bufferflags(usage: BufferUsage) -> vk::BufferUsageFlags {
    match usage {
        BufferUsage::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
        BufferUsage::Index => vk::BufferUsageFlags::INDEX_BUFFER,
        BufferUsage::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
    }
}

fn property_to_vk_property(property: Property) -> vk::MemoryPropertyFlags {
    match property {
        Property::HostVisible => vk::MemoryPropertyFlags::HOST_VISIBLE,
        Property::DeviceLocal => vk::MemoryPropertyFlags::DEVICE_LOCAL,
    }
}

impl CreateBuffer for Context {
    fn allocate(
        &self,
        property: Property,
        usage: BufferUsage,
        size: u64,
    ) -> Result<Box<dyn BufferApi>, BufferError> {
        let context = self;
        unsafe {
            let device_memory_properties = context
                .instance
                .get_physical_device_memory_properties(context.physical_device);
            // make sure we can always copy from and to a buffer
            let vk_usage = bitflag_to_bufferflags(usage)
                | vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::TRANSFER_DST;
            let vertex_input_buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size,
                usage: vk_usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 0,
                p_queue_family_indices: ptr::null(),
            };
            let vertex_input_buffer = context
                .device
                .create_buffer(&vertex_input_buffer_info, None)
                .unwrap();
            let vertex_input_buffer_memory_req = context
                .device
                .get_buffer_memory_requirements(vertex_input_buffer);
            let vertex_input_buffer_memory_index =
                find_memorytype_index(
                    &vertex_input_buffer_memory_req,
                    &device_memory_properties,
                    property_to_vk_property(property),
                ).expect("Unable to find suitable memorytype for the vertex buffer.");

            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                p_next: ptr::null(),
                allocation_size: vertex_input_buffer_memory_req.size,
                memory_type_index: vertex_input_buffer_memory_index,
            };
            let vertex_input_buffer_memory = context
                .device
                .allocate_memory(&vertex_buffer_allocate_info, None)
                .unwrap();
            context
                .device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
                .unwrap();
            let inner_buffer = BufferData {
                context: context.clone(),
                buffer: vertex_input_buffer,
                memory: vertex_input_buffer_memory,
                size,
            };
            Ok(Box::new(inner_buffer))
        }
    }
}
impl BufferApi for BufferData {
    fn size(&self) -> u64 {
        self.size
    }
    fn map_memory(&self) -> Result<*mut (), MappingError> {
        unsafe {
            let ptr = self
                .context
                .device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                .map_err(|_| MappingError::Failed)?;
            Ok(ptr as *mut ())
        }
    }
    fn unmap_memory(&self) {
        unsafe {
            self.context.device.unmap_memory(self.memory);
        }
    }
    // fn copy_to_device_local(&self) -> Result<Box<dyn BufferApi>, BufferError> {
    //     let context = &self.buffer.context;
    //     let dst_buffer =
    //         ImplBuffer::<T, DeviceLocal, Vulkan>::allocate(context, self.usage, self.buffer.len)?;
    //     let command_buffer = CommandBuffer::record(context, |command_buffer| unsafe {
    //         context.device.cmd_copy_buffer(
    //             command_buffer,
    //             self.buffer.buffer,
    //             dst_buffer.buffer.buffer,
    //             &[vk::BufferCopy {
    //                 src_offset: 0,
    //                 dst_offset: 0,
    //                 size: (self.buffer.len * size_of::<T>()) as _,
    //             }],
    //         );
    //     });
    //     context.present_queue.submit(
    //         context,
    //         &[vk::PipelineStageFlags::TOP_OF_PIPE],
    //         &[],
    //         &[],
    //         command_buffer,
    //     );
    //     Ok(dst_buffer)
    // }
}
// impl<T> HostVisibleBuffer<T, Vulkan> for ImplBuffer<T, HostVisible, Vulkan>
// where
//     T: Copy,
// {
//     fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
//     where
//         F: Fn(&mut [T]) -> R,
//     {
//         use std::slice::from_raw_parts_mut;
//         unsafe {
//             let byte_len = (self.buffer.len * size_of::<T>()) as u64;
//             let mapping_ptr = self
//                 .buffer
//                 .context
//                 .device
//                 .map_memory(self.buffer.memory, 0, byte_len, vk::MemoryMapFlags::empty())
//                 .map_err(|_| MappingError::Failed)?;
//             let slice = from_raw_parts_mut::<T>(mapping_ptr as *mut T, self.buffer.len);
//             let r = f(slice);
//             self.buffer.context.device.unmap_memory(self.buffer.memory);
//             Ok(r)
//         }
//     }

//     fn from_slice(
//         context: &Context<Vulkan>,
//         usage: BufferUsage,
//         data: &[T],
//     ) -> Result<Self, BufferError> {
//         let mut buffer = Self::allocate(context, usage, data.len())?;
//         buffer
//             .map_memory(|slice| slice.copy_from_slice(data))
//             .map_err(BufferError::MappingError)?;
//         Ok(buffer)
//     }
// }

/// helper function to find the correct memory index
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    // Try to find an exactly matching memory flag
    let best_suitable_index =
        find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
            property_flags == flags
        });
    if best_suitable_index.is_some() {
        return best_suitable_index;
    }
    // Otherwise find a memory flag that works
    find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
        property_flags & flags == flags
    })
}

fn find_memorytype_index_f<F: Fn(vk::MemoryPropertyFlags, vk::MemoryPropertyFlags) -> bool>(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
    f: F,
) -> Option<u32> {
    let mut memory_type_bits = memory_req.memory_type_bits;
    for (index, ref memory_type) in memory_prop.memory_types.iter().enumerate() {
        if memory_type_bits & 1 == 1 {
            if f(memory_type.property_flags, flags) {
                return Some(index as u32);
            }
        }
        memory_type_bits = memory_type_bits >> 1;
    }
    None
}
