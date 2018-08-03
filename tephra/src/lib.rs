#![feature(rust_2018_preview)]
extern crate thread_local_object;
pub extern crate ash;
extern crate failure;
extern crate serde;
extern crate enumflags;
#[macro_use]
extern crate enumflags_derive;
pub extern crate winit;
#[macro_use]
extern crate failure_derive;

pub mod backend;
pub mod buffer;
pub mod image;
pub mod context;

pub mod traits {
    use buffer::BufferApi;
    pub trait BackendApi
    where
        Self: Copy+ Clone + Sized + 'static,
    {
        type Buffer;
        type Context: Clone;
    }

}

pub mod errors {
    #[derive(Debug, Fail)]
    pub enum MappingError {
        #[fail(display = "Offset is out of range")]
        OutOfRange,
        #[fail(display = "Failed to map memory")]
        Failed
    }
    #[derive(Debug, Fail)]
    pub enum AllocationError {
        #[fail(display = "Unsupported memory type")]
        UnsupportedMemorytype,
    }
    #[derive(Debug, Fail)]
    pub enum BufferError {
        #[fail(display = "Allocation failed with: {}", _0)]
        AllocationError(AllocationError),
        // #[fail(display = "invalid toolchain name:")]
        // InvalidToolchainName,
        // #[fail(display = "unknown toolchain version: {}", version)]
        // UnknownToolchainVersion { version: String },
    }
}

pub struct Context<B: traits::BackendApi> {
    context: B::Context,
}
