use backend::BackendApi;
use context::Context;
use downcast::Downcast;
use slotmap::new_key_type;
use std::marker::PhantomData;
use std::mem::size_of;
new_key_type!(
    pub struct BufferHandle;
);

pub trait BufferApi {
    fn allocate(
        &self,
        property: Property,
        usage: BufferUsage,
        size: u64,
    ) -> Result<BufferHandle, BufferError>;
    fn destroy(&self, buffer: BufferHandle);
    unsafe fn map_memory(&self, buffer: BufferHandle) -> Result<*mut (), MappingError>;
    unsafe fn unmap_memory(&self, buffer: BufferHandle);
    unsafe fn size(&self, buffer: BufferHandle) -> u64;
}

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

//pub trait HostVisibleBuffer<T, Backend: BackendApi>
//where
//    Self: Sized,
//    T: Copy,
//{
//    //fn from_slice(
//        // context: &context::Context<Backend>,
//        // usage: BufferUsage,
//        // data: &[T],
//    // ) -> Result<Self, BufferError>;
//    // fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
//    // where
//        // F: Fn(&mut [T]) -> R;
//}

// impl_downcast!(BufferApi);

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
#[derive(Copy, Clone)]
pub struct Buffer<T> {
    pub _m: PhantomData<T>,
    pub buffer: BufferHandle,
}

impl<T: Copy> Buffer<T> {
    pub fn len(&self, ctx: &Context) -> u32 {
        unsafe { (ctx.size(self.buffer) / size_of::<T>() as u64) as u32 }
    }
    pub fn allocate(
        context: &Context,
        property: Property,
        usage: BufferUsage,
        elements: u64,
    ) -> Result<Self, BufferError> {
        let size = elements * size_of::<T>() as u64;
        let buffer = BufferApi::allocate(context.context.as_ref(), property, usage, size)?;
        Ok(Buffer {
            buffer,
            _m: PhantomData,
        })
    }

    pub fn update(&self, ctx: &Context, data: &[T]) -> Result<(), BufferError> {
        use std::slice::from_raw_parts_mut;
        unsafe {
            let mapping_ptr = ctx
                .map_memory(self.buffer)
                .map_err(BufferError::MappingError)?;
            let slice = unsafe { from_raw_parts_mut::<T>(mapping_ptr as *mut T, data.len()) };
            slice.copy_from_slice(data);
            ctx.unmap_memory(self.buffer);
            Ok(())
        }
    }

    pub fn from_slice(
        ctx: &Context,
        property: Property,
        usage: BufferUsage,
        data: &[T],
    ) -> Result<Self, BufferError> {
        use std::slice::from_raw_parts_mut;
        unsafe {
            let buffer = Self::allocate(ctx, property, usage, data.len() as u64)?;
            let mapping_ptr = ctx
                .map_memory(buffer.buffer)
                .map_err(BufferError::MappingError)?;
            let slice = unsafe { from_raw_parts_mut::<T>(mapping_ptr as *mut T, data.len()) };
            slice.copy_from_slice(data);
            ctx.unmap_memory(buffer.buffer);
            Ok(buffer)
        }
    }
}

// impl<T: Copy, Backend> Buffer<T, HostVisible, Backend>
// where
//     Backend: BackendApi,
//     ImplBuffer<T, HostVisible, Backend>: HostVisibleBuffer<T, Backend>,
// {
//     pub fn from_slice(
//         context: &context::Context<Backend>,
//         usage: BufferUsage,
//         data: &[T],
//     ) -> Result<Self, BufferError> {
//         HostVisibleBuffer::from_slice(context, usage, data)
//             .map(|impl_buffer| Buffer { impl_buffer })
//     }

//     pub fn map_memory<R, F>(&mut self, f: F) -> Result<R, MappingError>
//     where
//         F: Fn(&mut [T]) -> R,
//     {
//         HostVisibleBuffer::map_memory(&mut self.impl_buffer, f)
//     }
// }

// impl<T: Copy, Property, Backend> Buffer<T, Property, Backend>
// where
//     Backend: BackendApi,
//     Property: BufferProperty,
//     ImplBuffer<T, Property, Backend>: BufferApi<Backend, Item=T>,
// {
//     pub fn copy_to_device_local(&self) -> Result<Buffer<T, DeviceLocal, Backend>, BufferError> {
//         self.impl_buffer
//             .copy_to_device_local()
//             .map(|impl_buffer| Buffer { impl_buffer })
//     }
// }

#[derive(Copy, Clone)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
}
