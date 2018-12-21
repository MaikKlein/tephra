use crate::backend::BackendApi;
use crate::buffer::BufferHandle;
use crate::context::Context;
use crate::downcast::Downcast;
use crate::framegraph::{Compiled, Framegraph, Resource};
use parking_lot::{Mutex, MutexGuard};
use slotmap::new_key_type;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, Drop};
use std::sync::Arc;
new_key_type!(
    pub struct DescriptorHandle;
);
pub trait CreateDescriptor {}

pub trait CreatePool {
    fn create_pool(
        &self,
        alloc_size: u32,
        data: &[Binding<DescriptorType>],
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
    layout: Vec<Binding<DescriptorType>>,
    sizes: DescriptorSizes,
}

impl LinearPoolAllocator {
    pub fn new<T>(ctx: &Context) -> Self
    where
        T: DescriptorInfo,
    {
        let layout = T::layout();
        let sizes = DescriptorSizes::from_layout(&layout);
        LinearPoolAllocator {
            ctx: ctx.clone(),
            block_size: 50,
            pools: Vec::new(),
            layout,
            sizes,
        }
    }

    pub fn allocate_additional_pool(&mut self) {
        let pool = self
            .ctx
            .create_pool(self.block_size as u32, &self.layout, self.sizes);
        self.pools.push(pool);
    }

    pub fn reset(&mut self) {
        for pool in &mut self.pools {
            pool.inner.reset();
        }
    }
}

pub struct Allocator<'pool> {
    ctx: Context,
    pool: &'pool mut Pool,
    current_allocations: HashMap<TypeId, usize>,
}

impl<'a> Drop for Allocator<'a> {
    fn drop(&mut self) {
        self.pool.reset();
    }
}

impl<'pool> Allocator<'pool> {
    pub fn allocate<'alloc, T>(&'alloc mut self) -> Descriptor<'alloc, T>
    where
        T: DescriptorInfo + 'static,
    {
        let ctx = self.ctx.clone();
        let allocator = self
            .pool
            .allocators
            .entry(TypeId::of::<T>())
            .or_insert_with(|| LinearPoolAllocator::new::<T>(&ctx));
        let current_allocation = self
            .current_allocations
            .entry(TypeId::of::<T>())
            .or_insert(0);
        let allocator_index = *current_allocation / allocator.block_size;
        // If we don't have enough space, we need to allocate a new pool
        if allocator_index >= allocator.pools.len() {
            allocator.allocate_additional_pool();
        }
        let handle = allocator.pools[allocator_index].inner.create_descriptor();
        *current_allocation += 1;

        Descriptor {
            handle,
            _m: PhantomData,
        }
    }
}

pub struct Pool {
    ctx: Context,
    allocators: HashMap<TypeId, LinearPoolAllocator>,
}

impl Pool {
    pub fn new(ctx: &Context) -> Self {
        Pool {
            ctx: ctx.clone(),
            allocators: HashMap::new(),
        }
    }

    pub fn allocate<'a>(&'a mut self) -> Allocator<'a> {
        Allocator {
            ctx: self.ctx.clone(),
            pool: self,
            current_allocations: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        for allocator in self.allocators.values_mut() {
            allocator.reset();
        }
    }
}

pub trait CreateLayout {
    fn create_layout(&self, data: &[Binding<DescriptorType>]) -> NativeLayout;
}
pub trait LayoutApi: Downcast {
    //pub fn layout(&self) -> &[]
}
impl LayoutApi {
    pub fn downcast<B: BackendApi>(&self) -> &B::Layout {
        self.downcast_ref::<B::Layout>()
            .expect("Downcast Layout Vulkan")
    }
}
impl_downcast!(LayoutApi);

pub struct NativeLayout {
    pub inner: Box<dyn LayoutApi>,
}

pub struct Layout<T: DescriptorInfo> {
    pub inner_layout: NativeLayout,
    _m: PhantomData<T>,
}
impl<T> Layout<T>
where
    T: DescriptorInfo,
{
    pub fn new(ctx: &Context) -> Self {
        Layout {
            inner_layout: ctx.create_layout(&T::layout()),
            _m: PhantomData,
        }
    }
}
pub trait DescriptorApi {
    fn write(
        &self,
        handle: DescriptorHandle,
        data: &[Binding<DescriptorResource>],
        fg: &Framegraph<Compiled>,
    );
    fn create_descriptor(
        &self,
        data: &[Binding<DescriptorType>],
        sizes: DescriptorSizes,
    ) -> DescriptorHandle;
}

#[derive(Debug, Copy, Clone)]
pub struct DescriptorSizes {
    pub buffer: u32,
    pub storage: u32,
    pub images: u32,
}

impl DescriptorSizes {
    pub fn from_layout(layout: &[Binding<DescriptorType>]) -> Self {
        let sizes = DescriptorSizes {
            buffer: 0,
            storage: 0,
            images: 0,
        };
        layout.iter().fold(sizes, |mut acc, elem| {
            match elem.data {
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

#[derive(Debug)]
pub enum DescriptorType {
    Uniform,
    Storage,
}
pub enum DescriptorResource {
    Uniform(Resource<BufferHandle>),
    Storage(Resource<BufferHandle>),
}
#[derive(Debug)]
pub struct Binding<T> {
    pub binding: u32,
    pub data: T,
}

pub struct Descriptor<'a, T: DescriptorInfo> {
    pub handle: DescriptorHandle,
    _m: PhantomData<&'a T>,
}
impl<'a, T> Descriptor<'a, T>
where
    T: DescriptorInfo,
{
    pub fn update(&mut self, ctx: &Context, t: &'a T, fg: &Framegraph<Compiled>) {
        ctx.write(self.handle, &t.descriptor_data(), fg);
    }
}
