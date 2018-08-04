use traits::BackendApi;
use std::ops::Deref;

#[derive(Clone)]
pub struct Context<Backend: BackendApi> {
    pub context: Backend::Context,
}

impl<Backend> Deref for Context<Backend>
where Backend: BackendApi {
    type Target = Backend::Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
