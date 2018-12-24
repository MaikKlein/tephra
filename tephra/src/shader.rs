use crate::backend::BackendApi;
use crate::context::Context;
use crate::reflect;
use slotmap::new_key_type;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

new_key_type! {
    pub struct ShaderModule;
}
use crate::downcast::Downcast;

pub trait ShaderApi {
    unsafe fn create_shader(
        &self,
        bytes: &[u8],
    ) -> Result<ShaderModule, ShaderError>;
}

pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

pub trait GetShaderType {
    fn shader_type() -> ShaderType;
}

pub enum Vertex {}
impl GetShaderType for Vertex {
    fn shader_type() -> ShaderType {
        ShaderType::Vertex
    }
}

pub enum Fragment {}
impl GetShaderType for Fragment {
    fn shader_type() -> ShaderType {
        ShaderType::Fragment
    }
}
impl ShaderModule {
    pub unsafe fn load<P: AsRef<Path>>(
        ctx: &Context,
        p: P,
    ) -> Result<ShaderModule, ShaderError> {
        let file = File::open(p.as_ref()).map_err(ShaderError::IoError)?;
        let bytes: Vec<_> = file.bytes().filter_map(Result::ok).collect();
        ctx.create_shader(&bytes)
    }
}

// #[derive(Clone)]
// pub struct ShaderModule {
//     pub data: Arc<dyn ShaderApi>,
// }
// impl ShaderModule {
//     pub fn load<P: AsRef<Path>>(
//         context: &Context,
//         p: P,
//     ) -> Result<ShaderModule, ShaderError> {
//         let file = File::open(p.as_ref()).map_err(ShaderError::IoError)?;
//         let bytes: Vec<_> = file.bytes().filter_map(Result::ok).collect();
//         ShaderApi::create_shader(context.context.as_ref(), &bytes)
//     }
//     pub fn downcast<B: BackendApi>(&self) -> &B::Shader {
//         self.data
//             .downcast_ref::<B::Shader>()
//             .expect("Downcast Shader Vulkan")
//     }
// }

#[derive(Debug, Fail)]
pub enum ShaderError {
    #[fail(display = "Invalid shader")]
    Invalid,
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
}
