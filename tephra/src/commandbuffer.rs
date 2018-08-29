pub struct Graphics;
pub struct Compute;
pub trait CreateCommandbuffer<Type> {
    fn create_commandbuffer(&self) -> Commandbuffer<Type>;
}
pub trait CommandbufferApi<Type> {}
pub struct Commandbuffer<Type> {
    inner: Box<dyn CommandbufferApi<Type>>,
}
