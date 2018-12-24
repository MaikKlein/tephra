use super::Context;
use crate::shader::{ShaderApi, ShaderError, ShaderModule};
use ash::version::DeviceV1_0;
use ash::vk;
use std::ops::Drop;
use std::ptr;
use std::sync::Arc;
pub struct ShaderModuleData {
    pub shader_module: vk::ShaderModule,
}

impl ShaderApi for Context {
    unsafe fn create_shader(
        &self,
        bytes: &[u8],
    ) -> Result<ShaderModule, ShaderError> {
        let context = self;
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
        let shader_data = ShaderModuleData {
            shader_module,
        };
        Ok(self.shader_modules.insert(shader_data))
    }
}
