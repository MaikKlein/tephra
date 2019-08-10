use crate::context::Context;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

crate::new_typed_handle!(ShaderModule);

pub trait ShaderApi {
    unsafe fn create_shader(&self, bytes: &[u8]) -> Result<ShaderModule, ShaderError>;
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
    pub unsafe fn load<P: AsRef<Path>>(ctx: &Context, p: P) -> Result<ShaderModule, ShaderError> {
        let file = File::open(p.as_ref()).map_err(ShaderError::IoError)?;
        let bytes: Vec<_> = file.bytes().filter_map(Result::ok).collect();
        ctx.create_shader(&bytes)
    }
}

#[derive(Debug, Fail)]
pub enum ShaderError {
    #[fail(display = "Invalid shader")]
    Invalid,
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
}
