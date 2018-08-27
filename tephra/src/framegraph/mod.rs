use anymap::AnyMap;
use context::Context;
use framegraph::render_task::{Execute, RenderTask};
use image::{Image, ImageDesc, Resolution};
use petgraph::{self, Graph};
use render::Render;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
pub mod render_task;

pub struct Blackboard {
    any_map: AnyMap,
}
impl Blackboard {
    pub fn new() -> Blackboard {
        Blackboard {
            any_map: AnyMap::new(),
        }
    }
    pub fn add<T: 'static>(&mut self, t: T) {
        self.any_map.insert(t);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.any_map.get::<T>()
    }
}

#[derive(Debug)]
pub struct Resource<T> {
    _m: PhantomData<T>,
    pub handle: Handle,
    pub id: usize,
    pub name: &'static str,
}
impl<T> Copy for Resource<T> {}
impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Resource {
            id: self.id,
            name: self.name,
            handle: self.handle,
            _m: PhantomData,
        }
    }
}
impl<T> Resource<T> {
    pub fn new(name: &'static str, id: usize, handle: Handle) -> Self {
        Resource {
            id,
            name,
            handle,
            _m: PhantomData,
        }
    }
}

type Handle = petgraph::graph::NodeIndex;

pub struct TaskBuilder<'graph> {
    pass_handle: Handle,
    framegraph: &'graph mut Framegraph<Recording>,
}
impl<'graph> TaskBuilder<'graph> {
    pub fn create_image(&mut self, name: &'static str, desc: ImageDesc) -> Resource<Image> {
        self.framegraph.state.image_data.push(desc);
        let id = self.framegraph.state.image_data.len() - 1;
        Resource::new(name, id, self.pass_handle)
    }

    pub fn write<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Write,
            ty: ResourceType::Image,
        };
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, access);
        Resource::new(resource.name, resource.id, self.pass_handle)
    }

    pub fn read<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Read,
            ty: ResourceType::Image,
        };
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, access);
        resource
    }
}

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

#[derive(Debug, Copy, Clone)]
pub enum ResourceType {
    //Buffer,
    Image,
}

#[derive(Debug, Copy, Clone)]
pub struct Access {
    resource: usize,
    resource_access: ResourceAccess,
    ty: ResourceType,
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
    images: HashMap<usize, Image>,
    render: HashMap<Handle, Render>,
}

pub struct Recording {
    image_data: Vec<ImageDesc>,
    image_resource_map: HashMap<Handle, Vec<Resource<Image>>>,
}

pub struct Framegraph<T = Recording> {
    blackboard: Blackboard,
    state: T,
    graph: Graph<Pass, Access>,
    resources: Vec<()>,
    execute_fns: HashMap<Handle, Arc<dyn Execute>>,
}
pub trait GetResource<T> {
    fn get_resource(&self, resource: Resource<T>) -> &T;
}

impl GetResource<Image> for Framegraph<Compiled> {
    fn get_resource(&self, resource: Resource<Image>) -> &Image {
        self.state.images.get(&resource.id).expect("get image")
    }
}

impl Framegraph<Compiled> {
    pub fn get_resource<T>(&self, resource: Resource<T>) -> &T
    where
        Self: GetResource<T>,
    {
        GetResource::get_resource(self, resource)
    }
}

impl Framegraph {
    pub fn new(blackboard: Blackboard) -> Self {
        Framegraph {
            state: Recording {
                image_data: Vec::new(),
                image_resource_map: HashMap::new(),
            },
            graph: Graph::new(),
            resources: Vec::new(),
            execute_fns: HashMap::new(),
            blackboard,
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
        execute: fn(&Data, &Blackboard, &Render, &Framegraph<Compiled>),
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
            .image_resource_map
            .insert(pass_handle, image_resources);
        task
    }
    pub fn compile(self, resolution: Resolution, ctx: &Context) -> Framegraph<Compiled> {
        let images: HashMap<_, _> = self
            .state
            .image_data
            .iter()
            .enumerate()
            .map(|(id, image_desc)| {
                let image = Image::allocate(ctx, image_desc.clone());
                (id, image)
            })
            .collect();
        let render: HashMap<_, _> = self
            .state
            .image_resource_map
            .iter()
            .map(|(&handle, image_resources)| {
                let images: Vec<&Image> = image_resources
                    .iter()
                    .map(|&resource| images.get(&resource.id).expect("resource"))
                    .collect();
                (handle, Render::new(ctx, resolution, &images))
            })
            .collect();
        let state = Compiled { images, render };
        Framegraph {
            execute_fns: self.execute_fns,
            resources: self.resources,
            graph: self.graph,
            state,
            blackboard: self.blackboard,
        }
    }
}

impl Framegraph<Compiled> {
    // fn submission_order(&self) -> impl Iterator<Item=Handle> {
    //     (0..1)
    // }

    pub fn execute(&self) {
        use petgraph::visit::{Bfs, Walker};
        let bfs = Bfs::new(&self.graph, Handle::new(0));
        bfs.iter(&self.graph).for_each(|idx| {
            let execute = self.execute_fns.get(&idx).expect("renderpass");

            let render = self.state.render.get(&idx).expect("render");
            execute.execute(&self.blackboard, render, self);
        });
    }
    pub fn export_graphviz<P: AsRef<Path>>(&self, path: P) {
        use std::io::Write;
        let mut file = File::create(path.as_ref()).expect("path");
        let dot = petgraph::dot::Dot::with_config(&self.graph, &[]);
        write!(&mut file, "{}", dot);
    }
}
