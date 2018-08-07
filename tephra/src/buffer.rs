use backend::BackendApi;
use context;
use enumflags::BitFlags;
use std::marker::PhantomData;

#[derive(Debug, Fail)]
pub enum AllocationError {
    #[fail(display = "Unsupported memory type")]
    UnsupportedMemorytype,
}
#[derive(Debug, Fail)]
pub enum MappingError {
    #[fail(display = "Offset is out of range")]
    OutOfRange,
    #[fail(display = "Failed to map memory")]
    Failed,
}

#[derive(Debug, Fail)]
pub enum BufferError {
    #[fail(display = "Allocation failed with: {}", _0)]
    AllocationError(AllocationError),
    #[fail(display = "Mapping failed: {}", _0)]
    MappingError(MappingError),
}
pub enum HostVisible {}
pub enum DeviceLocal {}

pub trait HostVisibleBuffer<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn from_slice(
        context: &context::Context<Backend>,
        usage: BitFlags<BufferUsage>,
        data: &[T],
    ) -> Result<Self, BufferError>;
    fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
    where
        F: Fn(&mut [T]) -> R;
}

pub trait BufferApi<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn allocate(
        context: &context::Context<Backend>,
        usage: BitFlags<BufferUsage>,
        elements: usize,
    ) -> Result<Self, BufferError>;

    fn copy_to_device_local(&self) -> Result<ImplBuffer<T, DeviceLocal, Backend>, BufferError>;
}

pub trait BufferProperty {
    fn property() -> Property;
}

#[derive(Copy, Clone)]
pub enum Property {
    HostVisible,
    DeviceLocal,
}

impl BufferProperty for HostVisible {
    fn property() -> Property {
        Property::HostVisible
    }
}

impl BufferProperty for DeviceLocal {
    fn property() -> Property {
        Property::DeviceLocal
    }
}

pub struct ImplBuffer<T, Property, Backend>
where
    Backend: BackendApi,
    Property: BufferProperty,
{
    pub buffer: Backend::Buffer,
    pub usage: BitFlags<BufferUsage>,
    pub _m: PhantomData<T>,
    pub _property: PhantomData<Property>,
}

pub struct Buffer<T, Property, Backend>
where
    Property: BufferProperty,
    Backend: BackendApi,
{
    pub impl_buffer: ImplBuffer<T, Property, Backend>,
}

impl<T: Copy, Backend> Buffer<T, HostVisible, Backend>
where
    Backend: BackendApi,
    ImplBuffer<T, HostVisible, Backend>: HostVisibleBuffer<T, Backend>,
{
    pub fn from_slice(
        context: &context::Context<Backend>,
        usage: BitFlags<BufferUsage>,
        data: &[T],
    ) -> Result<Self, BufferError> {
        HostVisibleBuffer::from_slice(context, usage, data)
            .map(|impl_buffer| Buffer { impl_buffer })
    }

    pub fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
    where
        F: Fn(&mut [T]) -> R,
    {
        HostVisibleBuffer::map_memory(&mut self.impl_buffer, f)
    }
}

impl<T: Copy, Property, Backend> Buffer<T, Property, Backend>
where
    Backend: BackendApi,
    Property: BufferProperty,
    ImplBuffer<T, Property, Backend>: BufferApi<T, Backend>,
{
    pub fn copy_to_device_local(&self) -> Result<Buffer<T, DeviceLocal, Backend>, BufferError> {
        self.impl_buffer
            .copy_to_device_local()
            .map(|impl_buffer| Buffer { impl_buffer })
    }
}

#[derive(Copy, Clone, EnumFlags)]
#[repr(u32)]
pub enum BufferUsage {
    Vertex = 1 << 0,
    Index = 1 << 1,
    Uniform = 1 << 2,
}
