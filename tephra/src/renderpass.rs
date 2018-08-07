use backend::BackendApi;
use std::marker::PhantomData;

pub struct Renderpass<Backend: BackendApi> {
    inner: ImplRenderpass<Backend>,
}

pub struct ImplRenderpass<Backend: BackendApi> {
    _m: PhantomData<Backend>,
}
