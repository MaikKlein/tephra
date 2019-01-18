use super::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use tephra::descriptor::{
    Binding, CreatePool, DescriptorApi, DescriptorHandle, DescriptorResource, DescriptorSizes,
    DescriptorType, NativePool, PoolApi,
};
use tephra::framegraph::GetResource;
use tephra::framegraph::{Compiled, Framegraph};
pub struct Pool {
    pub ctx: Context,
    pub pool: vk::DescriptorPool,
    pub layouts: Vec<vk::DescriptorSetLayout>,
}

impl PoolApi for Pool {
    fn reset(&mut self) {
        unsafe {
            self.ctx
                .device
                .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty());
        }
    }
    fn create_descriptor(&self) -> DescriptorHandle {
        let desc_alloc_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.pool,
            descriptor_set_count: self.layouts.len() as u32,
            p_set_layouts: self.layouts.as_ptr(),
            ..Default::default()
        };
        let descriptor_set = unsafe {
            self.ctx
                .device
                .allocate_descriptor_sets(&desc_alloc_info)
                .unwrap()[0]
        };
        let inner = Descriptor { descriptor_set };
        self.ctx.descriptors.insert(inner)
    }
}
impl CreatePool for Context {
    fn create_pool(
        &self,
        alloc_size: u32,
        data: &[Binding<DescriptorType>],
        sizes: DescriptorSizes,
    ) -> NativePool {
        let layout_bindings: Vec<_> = data
            .iter()
            .map(|desc| {
                let ty = match desc.data {
                    DescriptorType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                    DescriptorType::Storage => vk::DescriptorType::STORAGE_BUFFER,
                };
                vk::DescriptorSetLayoutBinding {
                    binding: desc.binding,
                    descriptor_type: ty,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::ALL,
                    p_immutable_samplers: std::ptr::null(),
                }
            })
            .collect();
        let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: layout_bindings.len() as u32,
            p_bindings: layout_bindings.as_ptr(),
            ..Default::default()
        };

        let layouts = vec![unsafe {
            self.device
                .create_descriptor_set_layout(&descriptor_info, None)
                .unwrap()
        }];
        let mut pool_sizes = Vec::new();
        let buffer_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: sizes.buffer,
        };
        let storage_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: sizes.storage,
        };
        if storage_size.descriptor_count > 0 {
            pool_sizes.push(storage_size);
        }
        if buffer_size.descriptor_count > 0 {
            pool_sizes.push(buffer_size);
        }
        let image_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_IMAGE,
            descriptor_count: sizes.images,
        };
        if image_size.descriptor_count > 0 {
            pool_sizes.push(image_size);
        }
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo {
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            max_sets: alloc_size,
            ..Default::default()
        };
        let pool = unsafe {
            self.device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("create pool")
        };
        let inner = Pool {
            ctx: self.clone(),
            layouts,
            pool,
        };
        NativePool {
            inner: Box::new(inner),
        }
    }
}
pub struct Descriptor {
    pub descriptor_set: vk::DescriptorSet,
}

impl DescriptorApi for Context {
    fn create_descriptor(
        &self,
        _data: &[Binding<DescriptorType>],
        _sizes: DescriptorSizes,
    ) -> DescriptorHandle {
        unimplemented!()
    }
    fn write(
        &self,
        handle: DescriptorHandle,
        data: &[Binding<DescriptorResource>],
    ) {
        let descriptor = self.descriptors.get(handle);
        let buffer_infos: Vec<Binding<vk::DescriptorBufferInfo>> = data
            .iter()
            .map(|resource| match resource.data {
                DescriptorResource::Uniform(buffer) | DescriptorResource::Storage(buffer) => {
                    let generic_buffer = buffer;
                    let vkbuffer = self.buffers.get(generic_buffer);
                    let buffer_info = vk::DescriptorBufferInfo {
                        buffer: vkbuffer.buffer,
                        offset: 0,
                        range: vkbuffer.size,
                    };
                    Binding {
                        data: buffer_info,
                        binding: resource.binding,
                    }
                }
            })
            .collect();

        let writes: Vec<_> = buffer_infos
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let ty = match data[idx].data {
                    DescriptorResource::Uniform(_) => vk::DescriptorType::UNIFORM_BUFFER,
                    DescriptorResource::Storage(_) => vk::DescriptorType::STORAGE_BUFFER,
                };
                vk::WriteDescriptorSet {
                    s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                    p_next: std::ptr::null(),
                    dst_set: descriptor.descriptor_set,
                    dst_binding: info.binding,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: ty,
                    p_image_info: std::ptr::null(),
                    p_buffer_info: &info.data,
                    p_texel_buffer_view: std::ptr::null(),
                }
            })
            .collect();
        unsafe {
            self.device.update_descriptor_sets(&writes, &[]);
        }
    }
}
