use std::sync::Arc;
use backend::BackendApi;
use context::Context;
use std::fs::File;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::path::Path;

use downcast::Downcast;

pub trait CreateShader
{
    fn load(&self, bytes: &[u8]) -> Result<Shader, ShaderError>;
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

pub trait ShaderApi: Downcast {
}
impl_downcast!(ShaderApi);

#[derive(Clone)]
pub struct Shader {
    pub data: Arc<dyn ShaderApi>
}

impl Shader
{
    pub fn load<P: AsRef<Path>>(context: &Context, p: P) -> Result<Shader, ShaderError> {
        let file = File::open(p.as_ref()).map_err(ShaderError::IoError)?;
        let bytes: Vec<_> = file.bytes().filter_map(Result::ok).collect();
        CreateShader::load(context.context.as_ref(), &bytes)
    }
    pub fn downcast<B: BackendApi>(&self) -> &B::Shader {
        self.data.downcast_ref::<B::Shader>().expect("Downcast Shader Vulkan")
    }
}

#[derive(Debug, Fail)]
pub enum ShaderError {
    #[fail(display = "Invalid shader")]
    Invalid,
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
}
