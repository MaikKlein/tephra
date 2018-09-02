use buffer::{Buffer, BufferApi, GenericBuffer};
use commandbuffer::GraphicsCommandbuffer;
use context::Context;
use framegraph::render_task::{Execute, RenderTask};
use image::{Image, ImageApi, ImageDesc, Resolution};
use petgraph::{self, Graph};
use render::Render;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
pub mod blackboard;
pub mod render_task;
pub mod task_builder;
pub use self::blackboard::Blackboard;
use self::task_builder::TaskBuilder;

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

type Handle = petgraph::graph::NodeIndex;

#[derive(Debug, Copy, Clone)]
pub enum PassType {
    Graphics,
    Compute,
}
#[derive(Debug, Copy, Clone)]
pub struct Pass {
    name: &'static str,
    ty: PassType,
}

#[derive(Debug, Copy, Clone)]
pub enum ResourceAccess {
    Create,
    Read,
    Write,
}

pub enum ResourceType {
    Buffer(GenericBuffer),
    Image(Image),
}
impl ResourceType {
    pub fn as_buffer(&self) -> &GenericBuffer {
        match self {
            ResourceType::Buffer(buffer) => buffer,
            _ => panic!(""),
        }
    }
    pub fn as_image(&self) -> &Image {
        match self {
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

pub struct Compiled {
    render: HashMap<Handle, Render>,
}

pub struct Recording {
    image_data: Vec<(ResourceIndex, ImageDesc)>,
    frame_buffer_layout: HashMap<Handle, Vec<Resource<Image>>>,
}

pub struct Framegraph<T = Recording> {
    state: T,
    graph: Graph<Pass, Access>,
    resources: Vec<ResourceType>,
    execute_fns: HashMap<Handle, Arc<dyn Execute>>,
    pass_map: HashMap<(ResourceIndex, u32), Handle>,
}

pub trait GetResource<T> {
    fn get_resource(&self, resource: Resource<T>) -> &T;
}

impl<T> GetResource<Image> for Framegraph<T> {
    fn get_resource(&self, resource: Resource<Image>) -> &Image {
        self.resources[resource.id].as_image()
    }
}

impl<T> GetResource<GenericBuffer> for Framegraph<T> {
    fn get_resource(&self, resource: Resource<GenericBuffer>) -> &GenericBuffer {
        self.resources[resource.id].as_buffer()
    }
}
impl<T> Framegraph<T> {
    pub fn insert_pass_handle<D>(&mut self, resource: Resource<D>, handle: Handle) {
        self.pass_map
            .insert((resource.id, resource.version), handle);
    }
    pub fn get_pass_handle<D>(&self, resource: Resource<D>) -> Option<Handle> {
        self.pass_map.get(&(resource.id, resource.version)).cloned()
    }
}
impl Framegraph {
    pub fn add_resource(&mut self, ty: ResourceType) -> ResourceIndex {
        let id = self.resources.len();
        self.resources.push(ty);
        id
    }
    pub fn add_image(&mut self, image: Image) -> Resource<Image> {
        let id = self.add_resource(ResourceType::Image(image));
        Resource::new(id, 0)
    }
    pub fn add_buffer<T>(&mut self, buffer: Buffer<T>) -> Resource<GenericBuffer> {
        let id = self.add_resource(ResourceType::Buffer(buffer.buffer));
        Resource::new(id, 0)
    }
    pub fn new() -> Self {
        Framegraph {
            state: Recording {
                image_data: Vec::new(),
                frame_buffer_layout: HashMap::new(),
            },
            graph: Graph::new(),
            resources: Vec::new(),
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
    pub fn add_render_pass<Data, P, Setup>(
        &mut self,
        name: &'static str,
        setup: Setup,
        pass: P,
        execute: render_task::ExecuteFn<Data>,
    ) -> render_task::ARenderTask<Data>
    where
        Setup: Fn(&mut TaskBuilder) -> Data,
        P: Fn(&Data) -> Vec<Resource<Image>>,
        Data: 'static,
    {
        let (pass_handle, image_resources, task) = {
            let renderpass = Pass {
                name,
                ty: PassType::Graphics,
            };
            let pass_handle = self.graph.add_node(renderpass);
            let mut builder = TaskBuilder {
                pass_handle,
                framegraph: self,
            };
            let data = setup(&mut builder);
            let image_resources = pass(&data);
            let task = RenderTask { data, execute };
            (pass_handle, image_resources, Arc::new(task))
        };
        self.execute_fns.insert(pass_handle, task.clone());
        self.state
            .frame_buffer_layout
            .insert(pass_handle, image_resources);
        task
    }
    pub fn compile(mut self, resolution: Resolution, ctx: &Context) -> Framegraph<Compiled> {
        let images: Vec<_> = self
            .state
            .image_data
            .iter()
            .map(|(id, image_desc)| (*id, Image::allocate(ctx, image_desc.clone())))
            .collect();
        for (id, image) in images {
            self.resources.insert(id, ResourceType::Image(image));
        }
        let render: HashMap<_, _> = self
            .state
            .frame_buffer_layout
            .iter()
            .map(|(&handle, image_resources)| {
                let images: Vec<&Image> = image_resources
                    .iter()
                    .map(|&resource| self.get_resource(resource))
                    .collect();
                (handle, Render::new(ctx, resolution, &images))
            })
            .collect();
        let state = Compiled { render };
        Framegraph {
            execute_fns: self.execute_fns,
            resources: self.resources,
            graph: self.graph,
            state,
            pass_map: self.pass_map,
        }
    }
}

impl Framegraph<Compiled> {
    // fn submission_order(&self) -> impl Iterator<Item=Handle> {
    //     (0..1)
    // }

    pub fn execute(&self, blackboard: &Blackboard) {
        use petgraph::visit::{Bfs, Walker};
        let bfs = Bfs::new(&self.graph, Handle::new(0));
        bfs.iter(&self.graph).for_each(|idx| {
            let execute = self.execute_fns.get(&idx).expect("renderpass");

            let render = self.state.render.get(&idx).expect("render");
            let mut cmds = GraphicsCommandbuffer::new();
            execute.execute(blackboard, &mut cmds, self);
            render.execute_commands(self, &cmds.cmds);
        });
    }
    pub fn export_graphviz<P: AsRef<Path>>(&self, path: P) {
        use std::io::Write;
        let mut file = File::create(path.as_ref()).expect("path");
        let dot = petgraph::dot::Dot::with_config(&self.graph, &[]);
        write!(&mut file, "{}", dot);
    }
}
impl<T> Framegraph<T> {
    pub fn get_image(&self, id: ResourceIndex) -> &Image {
        self.resources[id].as_image()
    }
    pub fn get_buffer(&self, id: ResourceIndex) -> &GenericBuffer {
        self.resources[id].as_buffer()
    }
}

// pub struct ResourceStore {
//     images: Vec<Image>,
// }

// impl ResourceStore {
//     pub fn new() -> Self {
//         ResourceStore{
//             images: Vec::new(),
//         }
//     }
//     pub fn get_image(&self, id: usize) -> &Image {
//         &self.images[id]
//     }
// }
