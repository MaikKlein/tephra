use ash::version::{DeviceV1_0, InstanceV1_0, V1_0};
use ash::vk;
use ash::{Device, Entry, Instance};
use buffer::{self, BufferApi, BufferUsage, DeviceLocal, HostVisible, HostVisibleBuffer};
use enumflags::BitFlags;
use errors::{BufferError, MappingError};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, Drop};
use std::ptr;
use std::sync::Arc;
pub struct Vulkan;

impl crate::traits::BackendApi for Vulkan {
    type Buffer = Buffer;
    type Context = Context;
}

pub struct CommandPool {
    pool: vk::CommandPool,
}

#[derive(Clone)]
pub struct Context {
    pub inner_context: Arc<InnerContext>,
}

impl Deref for Context {
    type Target = InnerContext;
    fn deref(&self) -> &Self::Target {
        self.inner_context.deref()
    }
}

#[derive(Clone)]
pub struct InnerContext {
    pub entry: Entry<V1_0>,
    pub instance: Instance<V1_0>,
    pub device: Device<V1_0>,
    pub physical_device: vk::PhysicalDevice,
    //command_pool: CommandPool,
}
impl Drop for InnerContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
impl Context {
    pub fn new() -> Context {
        unimplemented!()
    }
}

pub struct Buffer {
    pub context: Context,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub len: usize,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_buffer(self.buffer, None);
            self.context.device.free_memory(self.memory, None);
        }
    }
}

fn bitflag_to_bufferflags(usage: BitFlags<BufferUsage>) -> vk::BufferUsageFlags {
    let mut flag = vk::BufferUsageFlags::default();
    if usage.contains(BufferUsage::Vertex) {
        flag |= vk::BufferUsageFlags::VERTEX_BUFFER;
    }
    // [TODO] Add all variants
    flag
}

impl Buffer {}

impl<T> HostVisibleBuffer<T, Vulkan> for buffer::Buffer<T, HostVisible, Vulkan>
where
    T: Copy,
{
    fn map_memory<R, F>(&mut self, mut f: F) -> Result<R, MappingError>
    where
        F: Fn(&mut [T]) -> R,
    {
        use std::slice::from_raw_parts_mut;
        unsafe {
            let byte_len = (self.buffer.len * size_of::<T>()) as u64;
            let mapping_ptr = self
                .buffer
                .context
                .device
                .map_memory(self.buffer.memory, 0, byte_len, vk::MemoryMapFlags::empty())
                .map_err(|_| MappingError::Failed)?;
            let slice = from_raw_parts_mut::<T>(mapping_ptr as *mut T, self.buffer.len);
            let r = f(slice);
            self.buffer.context.device.unmap_memory(self.buffer.memory);
            Ok(r)
        }
    }

    fn from_slice(
        context: &Context,
        usage: BitFlags<BufferUsage>,
        data: &[T],
    ) -> Result<Self, BufferError> {
        unsafe {
            let device_memory_properties = context
                .instance
                .get_physical_device_memory_properties(context.physical_device);
            let vk_usage = bitflag_to_bufferflags(usage);
            let vertex_input_buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: (data.len() * size_of::<T>()) as u64,
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
                vk::MemoryPropertyFlags::HOST_VISIBLE,
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
            let inner_buffer = Buffer {
                context: context.clone(),
                buffer: vertex_input_buffer,
                memory: vertex_input_buffer_memory,
                len: data.len(),
            };
            let mut buffer = buffer::Buffer {
                buffer: inner_buffer,
                usage,
                _m: PhantomData,
                _property: PhantomData,
            };
            buffer.map_memory(|slice| slice.copy_from_slice(data));
            Ok(buffer)
        }
    }
}

fn find_memorytype_index(
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
