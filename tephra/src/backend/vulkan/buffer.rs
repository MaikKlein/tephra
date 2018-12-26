use super::Context;
use crate::buffer::{BufferApi, BufferError, BufferHandle, BufferUsage, MappingError, Property};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use std::ptr;

impl BufferApi for Context {
    fn destroy(&self, buffer: BufferHandle) {
        let data = self.buffers.get(buffer);
        unsafe {
            self.device.destroy_buffer(data.buffer, None);
        }
    }
    fn allocate(
        &self,
        property: Property,
        usage: BufferUsage,
        size: u64,
    ) -> Result<BufferHandle, BufferError> {
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
            let vertex_input_buffer_memory_index = find_memorytype_index(
                &vertex_input_buffer_memory_req,
                &device_memory_properties,
                property_to_vk_property(property),
            )
            .expect("Unable to find suitable memorytype for the vertex buffer.");

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
                buffer: vertex_input_buffer,
                memory: vertex_input_buffer_memory,
                size,
            };
            Ok(self.buffers.insert(inner_buffer))
        }
    }

    unsafe fn map_memory(&self, buffer: BufferHandle) -> Result<*mut (), MappingError> {
        let data = self.buffers.get(buffer);
        let ptr = self
            .device
            .map_memory(data.memory, 0, data.size, vk::MemoryMapFlags::empty())
            .map_err(|_| MappingError::Failed)?;
        Ok(ptr as *mut ())
    }

    unsafe fn unmap_memory(&self, buffer: BufferHandle) {
        let data = self.buffers.get(buffer);
        self.device.unmap_memory(data.memory);
    }

    unsafe fn size(&self, buffer: BufferHandle) -> u64 {
        let data = self.buffers.get(buffer);
        data.size
    }
}
/// Vulkan specifc data
pub struct BufferData {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,
}

fn bitflag_to_bufferflags(usage: BufferUsage) -> vk::BufferUsageFlags {
    match usage {
        BufferUsage::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
        BufferUsage::Index => vk::BufferUsageFlags::INDEX_BUFFER,
        BufferUsage::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
        BufferUsage::Storage => vk::BufferUsageFlags::STORAGE_BUFFER,
    }
}

fn property_to_vk_property(property: Property) -> vk::MemoryPropertyFlags {
    match property {
        Property::HostVisible => vk::MemoryPropertyFlags::HOST_VISIBLE,
        Property::DeviceLocal => vk::MemoryPropertyFlags::DEVICE_LOCAL,
    }
}

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
