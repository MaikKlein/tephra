#[macro_use]
pub extern crate ash;
pub extern crate failure;
extern crate serde;
pub extern crate winit;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate downcast_rs as downcast;

pub mod buffer;
pub mod commandbuffer;
pub mod context;
pub mod descriptor;
//pub mod framegraph;
pub mod image;
pub mod passes;
pub mod pipeline;
pub mod reflect;
pub mod renderpass;
pub mod shader;
pub mod swapchain;
pub use failure::Error;
use parking_lot::RwLock;
#[derive(Copy, Clone, Default, Debug)]
pub struct Viewport {
    pub origin: (f32, f32),
    pub dimensions: (f32, f32),
    pub depth_range: (f32, f32),
}
use generational_arena::{Arena, Index};

pub trait TypedHandle {
    fn from_index(index: Index) -> Self;
    fn to_index(self) -> Index;
}

#[macro_export]
macro_rules! new_typed_handle {
    ($name: ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub struct $name(pub generational_arena::Index);
        impl crate::TypedHandle for $name {
            fn from_index(index: generational_arena::Index) -> Self {
                $name(index)
            }
            fn to_index(self) -> generational_arena::Index {
                self.0
            }
        }
    };
}
pub struct HandleMap<H, T> {
    map: RwLock<Arena<T>>,
    _marker: std::marker::PhantomData<H>,
}
impl<H, T> HandleMap<H, T>
where
    H: TypedHandle,
{
    pub fn insert(&self, data: T) -> H {
        H::from_index(self.map.write().insert(data))
    }

    pub fn new() -> Self {
        Self {
            map: RwLock::new(Arena::new()),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn is_valid(&self, key: H) -> bool {
        self.map.read().get(key.to_index()).is_some()
    }

    pub fn get(&self, key: H) -> parking_lot::MappedRwLockReadGuard<T> {
        parking_lot::RwLockReadGuard::map(self.map.read(), |data| data.get(key.to_index()).unwrap())
    }
}
