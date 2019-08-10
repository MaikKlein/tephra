use crate::{
    buffer::{Buffer, BufferHandle},
    commandbuffer::CommandList,
    context::Context,
    descriptor::Pool,
    framegraph::task_builder::TaskBuilder,
    image::{Image, ImageDesc},
    renderpass::{Framebuffer, Renderpass},
};
use petgraph::{self, Direction, Graph};
use std::{
    clone::Clone,
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    marker::PhantomData,
    path::Path,
};
pub mod blackboard;
pub mod render_task;
pub mod task_builder;
pub use self::blackboard::Blackboard;

pub trait Usage: Copy {
    fn usage() -> ResourceAccess;
}
impl Usage for Read {
    fn usage() -> ResourceAccess {
        ResourceAccess::Read
    }
}
impl Usage for Write {
    fn usage() -> ResourceAccess {
        ResourceAccess::Write
    }
}

pub trait Resource: Copy {
    type Type;
    fn id(&self) -> usize;
    fn version(&self) -> u32;
    fn usage(&self) -> ResourceAccess;
    fn increment<U: Usage>(self) -> ResourceBase<Self::Type, U>;
}

impl<T, U> Resource for ResourceBase<T, U>
where
    U: Usage,
{
    type Type = T;
    fn id(&self) -> usize {
        self.id
    }
    fn version(&self) -> u32 {
        self.version
    }
    fn usage(&self) -> ResourceAccess {
        U::usage()
    }
    fn increment<U1: Usage>(self) -> ResourceBase<Self::Type, U1> {
        self.increment()
    }
}

#[derive(Copy, Clone)]
pub enum Write {}
#[derive(Copy, Clone)]
pub enum Read {}
pub type WriteResource<T> = ResourceBase<T, Write>;
pub type ReadResource<T> = ResourceBase<T, Read>;
#[derive(Debug)]
pub struct ResourceBase<T, U> {
    _m: PhantomData<(T, U)>,
    version: u32,
    id: ResourceIndex,
}
pub type ResourceIndex = usize;

impl<T, U> Copy for ResourceBase<T, U> where U: Usage {}

impl<T, U> Clone for ResourceBase<T, U>
where
    U: Usage,
{
    fn clone(&self) -> Self {
        ResourceBase::new(self.id, self.version)
    }
}

impl<T, U> ResourceBase<T, U>
where
    U: Usage,
{
    pub fn increment<U1: Usage>(self) -> ResourceBase<T, U1> {
        ResourceBase::new(self.id, self.version + 1)
    }
    pub fn new(id: usize, version: u32) -> Self {
        ResourceBase {
            id,
            version,
            _m: PhantomData,
        }
    }
}

// impl<A, T> Resource<Buffer<T>, A> {
//     pub fn to_buffer_handle(self) -> Resource<BufferHandle, A> {
//         Resource::new(self.id, self.version)
//     }
// }

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
impl From<BufferHandle> for ResourceType {
    fn from(handle: BufferHandle) -> Self {
        ResourceType::Buffer(handle)
    }
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

impl<T: Copy> Resolve<Read> for Buffer<T> {
    type Target = Buffer<T>;
    fn resolve(&self, _registry: &Registry) -> Self::Target {
        *self
    }
}

impl<U, T> Resolve<U> for ResourceBase<Buffer<T>, U>
where
    U: Usage,
{
    type Target = Buffer<T>;
    fn resolve(&self, registry: &Registry) -> Self::Target {
        registry.get_buffer(*self)
    }
}
impl<U> Resolve<U> for ResourceBase<Framebuffer, U>
where
    U: Usage,
{
    type Target = Framebuffer;
    fn resolve(&self, registry: &Registry) -> Self::Target {
        registry.get_framebuffer(*self)
    }
}
pub trait Resolve<U> {
    type Target;
    fn resolve(&self, registry: &Registry) -> Self::Target;
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
    pub fn add_buffer<T>(&mut self, buffer: Buffer<T>) -> WriteResource<Buffer<T>> {
        let id = self.reserve_index();
        self.resources
            .insert(id, ResourceType::Buffer(buffer.buffer));
        WriteResource::new(id, 0)
    }
    pub fn add_image(&mut self, image: Image) -> WriteResource<Image> {
        let id = self.reserve_index();
        self.resources.insert(id, ResourceType::Image(image));
        WriteResource::new(id, 0)
    }
    pub fn get_framebuffer(&self, resource: impl Resource<Type = Framebuffer>) -> Framebuffer {
        match self.resources[&resource.id()] {
            ResourceType::Framebuffer(fb) => fb,
            _ => unreachable!(),
        }
    }
    pub fn get_buffer<T>(&self, resource: impl Resource<Type = Buffer<T>>) -> Buffer<T> {
        match self.resources[&resource.id()] {
            ResourceType::Buffer(buffer) => Buffer {
                _m: PhantomData,
                buffer,
            },
            _ => unreachable!(),
        }
    }
    pub fn get_image(&self, resource: impl Resource<Type = Image>) -> Image {
        match self.resources[&resource.id()] {
            ResourceType::Image(image) => image,
            _ => unreachable!(),
        }
    }
}
pub struct Compiled {}

pub struct Recording {
    image_data: Vec<(ResourceIndex, ImageDesc)>,
    framebuffer_data: Vec<(ResourceIndex, (Renderpass, Vec<WriteResource<Image>>))>,
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
        Fn(&Registry, &Blackboard, &mut CommandList) + 'static
}

pub struct Framegraph<T = Recording> {
    pool: Pool,
    pub ctx: Context,
    execute_fns: HashMap<Handle, Box<dyn ExecuteFn>>,
    state: T,
    graph: Graph<Pass, Access>,
    pub registry: Registry,
    pass_map: HashMap<(ResourceIndex, u32), Handle>,
}

// pub trait GetResource<T> {
//     fn get_resource(&self, resource: Resource<T>) -> T;
// }

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
    pub fn insert_pass_handle(&mut self, resource: impl Resource, handle: Handle) {
        self.pass_map
            .insert((resource.id(), resource.version()), handle);
    }
    pub fn get_pass_handle(&self, resource: impl Resource) -> Option<Handle> {
        self.pass_map
            .get(&(resource.id(), resource.version()))
            .cloned()
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
    pub fn new(ctx: &Context) -> Self {
        let pool = Pool::new(ctx);
        Framegraph {
            pool,
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
    pub fn add_pass<Setup, Execute, P>(&mut self, name: &'static str, mut setup: Setup) -> P
    where
        Setup: FnMut(&mut TaskBuilder<'_>) -> (P, Execute),
        Execute: Fn(&Registry, &Blackboard, &mut CommandList) + 'static,
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
            pool: self.pool,
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

    pub unsafe fn execute(&mut self, blackboard: &Blackboard) {
        let submission_order = self.submission_order();
        let mut command_list = CommandList::new();
        for idx in submission_order {
            // TODO: Improve pass execution
            let execute = self.execute_fns.get(&idx).unwrap();
            execute(&self.registry, blackboard, &mut command_list);
        }
        self.ctx.submit_commands(&mut self.pool, &command_list);
        self.pool.reset();
    }
    pub fn export_graphviz<P: AsRef<Path>>(&self, path: P) {
        use std::io::Write;
        let mut file = File::create(path.as_ref()).expect("path");
        let dot = petgraph::dot::Dot::with_config(&self.graph, &[]);
        write!(&mut file, "{}", dot);
    }
}

pub struct PassRunner {
    execute_fn: Option<Box<dyn ExecuteFn>>,
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
