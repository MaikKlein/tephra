use backend::BackendApi;
use context::Context;
use std::fs::File;
use std::io::{self, Read};
use std::marker::PhantomData;
use std::path::Path;

pub trait ShaderApi<Type, Backend>
where
    Self: Sized,
    Backend: BackendApi,
    Type: GetShaderType,
{
    fn load(context: &Context<Backend>, bytes: &[u8]) -> Result<Self, ShaderError>;
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

pub struct Shader<T: GetShaderType, Backend: BackendApi> {
    _shader_type: PhantomData<T>,
    shader_data: Backend::Shader,
}

impl<T, Backend> Shader<T, Backend>
where
    T: GetShaderType,
    Backend: BackendApi,
    Backend::Shader: ShaderApi<T, Backend>,
{
    pub fn load<P: AsRef<Path>>(context: &Context<Backend>, p: P) -> Result<Self, ShaderError> {
        let file = File::open(p.as_ref()).map_err(ShaderError::IoError)?;
        let bytes: Vec<_> = file.bytes().filter_map(Result::ok).collect();
        let shader_data = Backend::Shader::load(context, &bytes)?;
        let shader = Shader {
            shader_data,
            _shader_type: PhantomData,
        };
        Ok(shader)
    }
}

#[derive(Debug, Fail)]
pub enum ShaderError {
    #[fail(display = "Invalid shader")]
    Invalid,
    #[fail(display = "IO error {}", _0)]
    IoError(io::Error),
}
// pub type VertexShader = Shader<Vertex>;
// pub type FragmentShader = Shader<Fragment>;
