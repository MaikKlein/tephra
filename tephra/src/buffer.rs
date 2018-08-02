use errors::{MappingError, BufferError };
use std::marker::PhantomData;
use traits::BackendApi;

pub enum HostVisible {}
pub enum DeviceLocal {}

pub trait HostVisibleBuffer<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn from_slice(context: &Backend::Context, usage: BufferUsage, data: &[T]) -> Result<Self, BufferError>;
    fn map_memory<R, F>(&mut self, mut f: F) -> Result<R, MappingError>
        where F: Fn(&mut [T]) -> R;
}

pub trait BufferApi<T, Backend: BackendApi>
where
    Self: Sized,
    T: Copy,
{
    fn copy_to_device_local(&self) -> Buffer<T, DeviceLocal, Backend>;
}

pub struct Buffer<T, Property, Backend: BackendApi> {
    pub context: Backend::Context,
    pub buffer: Backend::Buffer,
    pub usage: BufferUsage,
    pub _m: PhantomData<T>,
    pub _property: PhantomData<Property>,
}

#[derive(Copy, Clone, EnumFlags)]
#[repr(u32)]
pub enum BufferUsage {
    Vertex = 1 << 0,
    Index = 1 << 1,
    Uniform = 1 << 2,
}
