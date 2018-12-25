use crate::{
    buffer::{Buffer, BufferApi, BufferHandle},
    commandbuffer::CommandList,
    context::Context,
    descriptor::{Allocator, Layout, NativeLayout, Pool},
    framegraph::task_builder::{deferred, TaskBuilder},
    image::{Image, ImageApi, ImageDesc, Resolution},
    pipeline::GraphicsPipeline,
    renderpass::{Framebuffer, Renderpass, RenderpassApi, RenderpassState},
};
use petgraph::Direction;
use petgraph::{self, Graph};
use std::clone::Clone;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
pub mod blackboard;
pub mod render_task;
pub mod task_builder;
pub use self::blackboard::Blackboard;

pub trait ResourceBase {}
#[derive(Debug)]
pub struct Resource<T> {
    _m: PhantomData<T>,
    pub id: usize,
    pub version: u32,
}
impl<T> ResourceBase for Resource<T> {}
pub type ResourceIndex = usize;

impl<T> Copy for Resource<T> {}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Resource {
            id: self.id,
            version: self.version,
            _m: PhantomData,
        }
    }
}
impl<T> Resource<T> {
    pub fn new(id: usize, version: u32) -> Self {
        Resource {
            id,
            version,
            _m: PhantomData,
        }
    }
}

impl<T> Resource<Buffer<T>> {
    pub fn to_buffer_handle(self) -> Resource<BufferHandle> {
        Resource {
            _m: PhantomData,
            id: self.id,
            version: self.version,
        }
    }
}

type Handle = petgraph::graph::NodeIndex;

#[derive(Debug, Copy, Clone)]
pub enum PassType {
    Graphics,
    Compute,
}
#[derive(Debug, Copy, Clone)]
pub struct Pass {
    name: &'static str,
}

#[derive(Debug, Copy, Clone)]
pub enum ResourceAccess {
    Create,
    Read,
    Write,
}

pub enum ResourceType {
    Buffer(BufferHandle),
    Image(Image),
    Framebuffer(Framebuffer),
}
impl ResourceType {
    pub fn as_buffer(&self) -> BufferHandle {
        match *self {
            ResourceType::Buffer(buffer) => buffer,
            _ => panic!(""),
        }
    }
    pub fn as_image(&self) -> Image {
        match *self {
            ResourceType::Image(image) => image,
            _ => panic!(""),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Access {
    resource: usize,
    resource_access: ResourceAccess,
}

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

pub struct Registry {
    resources: HashMap<ResourceIndex, ResourceType>,
    free_id: usize,
}
impl Registry {
    pub fn new() -> Self {
        Registry {
            resources: HashMap::new(),
            free_id: 0,
        }
    }
    pub fn reserve_index(&mut self) -> ResourceIndex {
        let id = self.free_id;
        self.free_id += 1;
        id
    }
    pub fn add_buffer<T>(&mut self, buffer: Buffer<T>) -> Resource<Buffer<T>> {
        let id = self.reserve_index();
        self.resources
            .insert(id, ResourceType::Buffer(buffer.buffer));
        Resource::new(id, 0)
    }
    pub fn add_image(&mut self, image: Image) -> Resource<Image> {
        let id = self.reserve_index();
        self.resources.insert(id, ResourceType::Image(image));
        Resource::new(id, 0)
    }
    pub fn get_framebuffer(&self, resource: Resource<Framebuffer>) -> Framebuffer {
        match self.resources[&resource.id] {
            ResourceType::Framebuffer(fb) => fb,
            _ => unreachable!(),
        }
    }
    pub fn get_buffer(&self, resource: Resource<BufferHandle>) -> BufferHandle {
        match self.resources[&resource.id] {
            ResourceType::Buffer(buffer) => buffer,
            _ => unreachable!(),
        }
    }
    pub fn get_image(&self, resource: Resource<Image>) -> Image {
        match self.resources[&resource.id] {
            ResourceType::Image(image) => image,
            _ => unreachable!(),
        }
    }
}
pub struct Compiled {}

pub struct Recording {
    image_data: Vec<(ResourceIndex, ImageDesc)>,
    framebuffer_data: Vec<(ResourceIndex, (Renderpass, Vec<Resource<Image>>))>,
}

/// Rust doesn't have type alias yet, so this is a work around
macro_rules! define_fn {
    (pub type $name: ident = $($tts:tt)*) => {
        pub trait $name: $($tts)* {}
        impl<T> $name for T
        where T: $($tts)* {

        }
    }
}
define_fn! {
    pub type ExecuteFn =
        Fn(&Framegraph<Compiled>, &Blackboard, &mut Allocator) -> CommandList + 'static
}

pub struct Framegraph<T = Recording> {
    pub ctx: Context,
    execute_fns: HashMap<Handle, Box<ExecuteFn>>,
    state: T,
    graph: Graph<Pass, Access>,
    pub registry: Registry,
    pass_map: HashMap<(ResourceIndex, u32), Handle>,
}

pub trait GetResource<T> {
    fn get_resource(&self, resource: Resource<T>) -> T;
}

// impl<T> GetResource<Image> for Framegraph<T> {
//     fn get_resource(&self, resource: Resource<Image>) -> Image {
//         self.resources[resource.id].as_image()
//     }
// }

// impl<T> GetResource<BufferHandle> for Framegraph<T> {
//     fn get_resource(&self, resource: Resource<BufferHandle>) -> BufferHandle {
//         self.resources[resource.id].as_buffer()
//     }
// }
// impl<D, T> GetResource<Buffer<D>> for Framegraph<T> {
//     fn get_resource(&self, resource: Resource<Buffer<D>>) -> Buffer<D> {
//         Buffer {
//             buffer: self.resources[resource.id].as_buffer(),
//             _m: PhantomData,
//         }
//     }
// }
impl<T> Framegraph<T> {
    pub fn insert_pass_handle<D>(&mut self, resource: Resource<D>, handle: Handle) {
        self.pass_map
            .insert((resource.id, resource.version), handle);
    }
    pub fn get_pass_handle<D>(&self, resource: Resource<D>) -> Option<Handle> {
        self.pass_map.get(&(resource.id, resource.version)).cloned()
    }
}
impl<T> Framegraph<T> {
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
}
impl Framegraph {
    // pub fn add_resource(&mut self, ty: ResourceType) -> ResourceIndex {
    //     let id = self.resources.len();
    //     self.resources.push(ty);
    //     id
    // }
    // pub fn add_image(&mut self, image: Image) -> Resource<Image> {
    //     let id = self.add_resource(ResourceType::Image(image));
    //     Resource::new(id, 0)
    // }
    // pub fn add_buffer<T>(&mut self, buffer: Buffer<T>) -> Resource<Buffer<T>> {
    //     let id = self.add_resource(ResourceType::Buffer(buffer.buffer));
    //     Resource::new(id, 0)
    // }
    pub fn new(ctx: &Context) -> Self {
        Framegraph {
            ctx: ctx.clone(),
            state: Recording {
                image_data: Vec::new(),
                framebuffer_data: Vec::new(),
            },
            graph: Graph::new(),
            registry: Registry::new(),
            execute_fns: HashMap::new(),
            pass_map: HashMap::new(),
        }
    }
    // pub fn add_compute_pass<Data, P, Setup>(
    //     &mut self,
    //     name: &'static str,
    //     setup: Setup,
    //     pass: P,
    //     execute: fn(&Data, &Blackboard, &Render, &Framegraph<Compiled>),
    // ) -> render_task::ARenderTask<Data> {
    // where
    //     Setup: Fn(&mut TaskBuilder) -> Data,
    //     P: Fn(&Data) -> Vec<Resource<Image>>,
    //     Data: 'static,
    // {
    //     unimplemented!()
    // }
    // pub fn add_render_pass<Input, P, Setup>(
    //     &mut self,
    //     name: &'static str,
    //     setup: Setup,
    //     pass: P,
    //     execute: render_task::ExecuteFn<Input>,
    // ) -> render_task::ARenderTask<Input>
    // where
    //     Setup: Fn(&mut TaskBuilder<'_, 'graph>) -> Input,
    //     P: Fn(&Input) -> Vec<Resource<Image>>,
    //     Input: 'static,
    // {
    //     let (pass_handle, image_resources, task) = {
    //         let renderpass = Pass {
    //             name,
    //             ty: PassType::Graphics,
    //         };
    //         let pass_handle = self.graph.add_node(renderpass);
    //         let input = {
    //             let mut builder = TaskBuilder {
    //                 pass_handle,
    //                 framegraph: self,
    //             };
    //             setup(&mut builder)
    //         };
    //         let image_resources = pass(&input);
    //         let task = RenderTask { data: input, execute };
    //         (pass_handle, image_resources, Arc::new(task))
    //     };
    //     self.execute_fns.insert(pass_handle, task.clone());
    //     self.state
    //         .frame_buffer_layout
    //         .insert(pass_handle, image_resources);
    //     task
    // }
    // pub fn add_compute_pass<F, P>(&mut self, name: &'static str, mut f: F) -> Arc<P>
    // where
    //     F: FnMut(&mut TaskBuilder<'_>) -> P,
    //     P: Computepass + 'static,
    // {
    //     let layout = Layout::<P::Layout>::new(&self.ctx);
    //     let (pass_handle, task) = {
    //         let renderpass = Pass {
    //             name,
    //             ty: PassType::Compute,
    //         };
    //         let pass_handle = self.graph.add_node(renderpass);
    //         let renderpass = {
    //             let mut builder = TaskBuilder {
    //                 pass_handle,
    //                 framegraph: self,
    //             };
    //             f(&mut builder)
    //         };
    //         (pass_handle, Arc::new(renderpass))
    //     };
    //     self.execute_compute.insert(pass_handle, task.clone());
    //     self.state.layouts.insert(pass_handle, layout.inner_layout);
    //     task
    // }
    // pub fn add_render_pass<F, P>(&mut self, name: &'static str, mut f: F) -> Arc<P>
    // where
    //     F: FnMut(&mut TaskBuilder<'_>) -> P,
    //     P: Renderpass + 'static,
    // {
    //     let layout = Layout::<P::Layout>::new(&self.ctx);
    //     let (pass_handle, image_resources, task) = {
    //         let renderpass = Pass {
    //             name,
    //             ty: PassType::Graphics,
    //         };
    //         let pass_handle = self.graph.add_node(renderpass);
    //         let renderpass = {
    //             let mut builder = TaskBuilder {
    //                 pass_handle,
    //                 framegraph: self,
    //             };
    //             f(&mut builder)
    //         };
    //         let image_resources = renderpass.framebuffer();
    //         (pass_handle, image_resources, Arc::new(renderpass))
    //     };
    //     self.execute_fns.insert(pass_handle, task.clone());
    //     self.state
    //         .frame_buffer_layout
    //         .insert(pass_handle, image_resources);
    //     self.state.layouts.insert(pass_handle, layout.inner_layout);
    //     task
    // }
    pub fn add_pass<Setup, Execute, P>(&mut self, name: &'static str, mut setup: Setup) -> P
    where
        Setup: FnMut(&mut TaskBuilder<'_>) -> (P, Execute),
        Execute:
            Fn(&Framegraph<Compiled>, &Blackboard, &mut Allocator<'_>) -> CommandList + 'static,
    {
        let (pass_handle, execute, data) = {
            let pass = Pass { name };
            let pass_handle = self.graph.add_node(pass);
            let (data, execute) = {
                let mut builder = TaskBuilder {
                    pass_handle,
                    framegraph: self,
                };
                setup(&mut builder)
            };
            (pass_handle, Box::new(execute), data)
        };
        self.execute_fns.insert(pass_handle, execute);
        data
    }
    pub fn compile(mut self) -> Framegraph<Compiled> {
        unsafe {
            for (id, image_desc) in &self.state.image_data {
                let image = Image::allocate(&self.ctx, image_desc.clone());
                self.registry
                    .resources
                    .insert(*id, ResourceType::Image(image));
            }
            for (id, (renderpass, images)) in &self.state.framebuffer_data {
                let images: Vec<_> = images
                    .iter()
                    .map(|&image_resource| self.registry.get_image(image_resource))
                    .collect();
                let framebuffer = self.ctx.create_framebuffer(*renderpass, &images);
                self.registry
                    .resources
                    .insert(*id, ResourceType::Framebuffer(framebuffer));
            }
        }

        Framegraph {
            ctx: self.ctx,
            execute_fns: self.execute_fns,
            registry: self.registry,
            graph: self.graph,
            state: Compiled {},
            pass_map: self.pass_map,
        }
    }
}

impl Framegraph<Compiled> {
    /// Calculates the submission order of all the passes
    fn submission_order(&self) -> impl Iterator<Item = Handle> {
        let mut submission = Vec::new();
        let mut cache = HashSet::new();

        // FIXME: Find real backbuffers. This is just a workaround because
        // there are no backbuffers yet.
        let backbuffer = self
            .graph
            .node_indices()
            .find(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .count()
                    == 0
            })
            .expect("Unable to find backbuffer");
        // We start from the backbuffer and traverse the graph backwards. After
        // we have collected all the indices of the passes
        self.record_submission(backbuffer, &mut submission, &mut cache);
        submission.into_iter()
    }

    fn record_submission(
        &self,
        node: Handle,
        submission: &mut Vec<Handle>,
        cache: &mut HashSet<Handle>,
    ) {
        self.graph
            .neighbors_directed(node, Direction::Incoming)
            .for_each(|neighbor| {
                self.record_submission(neighbor, submission, cache);
            });
        if !cache.contains(&node) {
            submission.push(node);
            cache.insert(node);
        }
    }

    pub fn execute(&mut self, blackboard: &Blackboard) {
        let mut pool = Pool::new(&self.ctx);
        let mut allocator = pool.allocate();
        self.submission_order().for_each(|idx| {
            // TODO: Improve pass execution
            let execute = self.execute_fns.get(&idx).unwrap();
            execute(self, blackboard, &mut allocator);
        });
    }
    pub fn export_graphviz<P: AsRef<Path>>(&self, path: P) {
        use std::io::Write;
        let mut file = File::create(path.as_ref()).expect("path");
        let dot = petgraph::dot::Dot::with_config(&self.graph, &[]);
        write!(&mut file, "{}", dot);
    }
}

pub struct PassRunner {
    execute_fn: Option<Box<ExecuteFn>>,
}
// impl PassRunner {
//     pub fn execute(mut self, f: ExecuteFn) {

//     }

// }
// impl<T> Framegraph<T> {
//     pub fn get_image(&self, id: ResourceIndex) -> Image {
//         self.resources[id].as_image()
//     }
//     pub fn get_buffer(&self, id: ResourceIndex) -> BufferHandle {
//         self.resources[id].as_buffer()
//     }
// }
