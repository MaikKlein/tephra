use crate::{
    buffer::BufferHandle,
    commandbuffer::{ShaderArguments, ShaderView, ShaderViews},
    context::Context,
};
use slotmap::new_key_type;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Drop;
new_key_type!(
    pub struct DescriptorHandle;
);
pub trait CreateDescriptor {}

pub trait CreatePool {
    fn create_pool(
        &self,
        alloc_size: u32,
        data: &[ShaderView],
        sizes: DescriptorSizes,
    ) -> NativePool;
}

pub trait PoolApi {
    fn create_descriptor(&self) -> DescriptorHandle;
    fn reset(&mut self);
}

pub struct NativePool {
    pub inner: Box<dyn PoolApi>,
}

pub struct LinearPoolAllocator {
    ctx: Context,
    block_size: usize,
    pools: Vec<NativePool>,
    // Infos
    views: ShaderViews,
    sizes: DescriptorSizes,
    current_allocations: usize,
}

impl LinearPoolAllocator {
    pub fn new(ctx: &Context, views: ShaderViews) -> Self {
        let sizes = DescriptorSizes::from_views(&views);
        LinearPoolAllocator {
            ctx: ctx.clone(),
            block_size: 50,
            pools: Vec::new(),
            views,
            sizes,
            current_allocations: 0,
        }
    }

    pub fn allocate_additional_pool(&mut self) {
        println!("allo {:?}", self.views);
        println!("allo {:?}", self.sizes);
        let pool = self
            .ctx
            .create_pool(self.block_size as u32, &self.views, self.sizes);
        self.pools.push(pool);
    }

    pub fn reset(&mut self) {
        for pool in &mut self.pools {
            pool.inner.reset();
            self.current_allocations = 0;
        }
    }
}

pub struct Pool {
    ctx: Context,
    allocators: HashMap<ShaderViews, LinearPoolAllocator>,
}

impl Pool {
    pub fn new(ctx: &Context) -> Self {
        Pool {
            ctx: ctx.clone(),
            allocators: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, data: &ShaderArguments) -> DescriptorHandle {
        println!("Allocate ");
        let ctx = self.ctx.clone();
        let allocator = self
            .allocators
            .entry(data.views.clone())
            .or_insert_with(|| LinearPoolAllocator::new(&ctx, data.views.clone()));
        let allocator_index = allocator.current_allocations / allocator.block_size;
        // If we don't have enough space, we need to allocate a new pool
        if allocator_index >= allocator.pools.len() {
            allocator.allocate_additional_pool();
        }
        let handle = allocator.pools[allocator_index].inner.create_descriptor();
        ctx.write(handle, &data);
        allocator.current_allocations += 1;
        handle
    }

    pub fn reset(&mut self) {
        for allocator in self.allocators.values_mut() {
            allocator.reset();
        }
    }
}

pub trait DescriptorApi {
    fn write(&self, handle: DescriptorHandle, data: &ShaderArguments);
}

#[derive(Debug, Copy, Clone)]
pub struct DescriptorSizes {
    pub buffer: u32,
    pub storage: u32,
    pub images: u32,
}

impl DescriptorSizes {
    pub fn from_views(views: &[ShaderView]) -> Self {
        let sizes = DescriptorSizes {
            buffer: 0,
            storage: 0,
            images: 0,
        };
        views.iter().fold(sizes, |mut acc, elem| {
            match elem.ty {
                DescriptorType::Uniform => acc.buffer += 1,
                DescriptorType::Storage => acc.storage += 1,
            }
            acc
        })
    }
}

pub trait DescriptorInfo
where
    Self: 'static,
{
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>>;
    fn layout() -> Vec<Binding<DescriptorType>>;
}
impl DescriptorInfo for () {
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>> {
        Vec::new()
    }
    fn layout() -> Vec<Binding<DescriptorType>> {
        Vec::new()
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum DescriptorType {
    Uniform,
    Storage,
}
pub enum DescriptorResource {
    Uniform(BufferHandle),
    Storage(BufferHandle),
}
#[derive(Debug)]
pub struct Binding<T> {
    pub binding: u32,
    pub data: T,
}
