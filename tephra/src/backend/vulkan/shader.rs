use super::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use shader::{CreateShader, ShaderApi, ShaderError, ShaderModule};
use std::ops::Drop;
use std::ptr;
use std::sync::Arc;
pub struct ShaderData {
    context: Context,
    pub shader_module: vk::ShaderModule,
}

impl Drop for ShaderData {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .destroy_shader_module(self.shader_module, None);
        }
    }
}
impl ShaderApi for ShaderData {}
impl CreateShader for Context {
    fn load(&self, bytes: &[u8]) -> Result<ShaderModule, ShaderError> {
        let context = self;
        unsafe {
            let shader_info = vk::ShaderModuleCreateInfo {
                s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                code_size: bytes.len(),
                p_code: bytes.as_ptr() as *const u32,
            };
            let shader_module = context
                .device
                .create_shader_module(&shader_info, None)
                .expect("Vertex shader module error");
            let shader_data = ShaderData {
                context: context.clone(),
                shader_module,
            };
            let shader = ShaderModule {
                data: Arc::new(shader_data),
            };
            Ok(shader)
        }
    }
}
