use anymap::AnyMap;
use context::Context;
use framegraph::render_task::{Execute, RenderTask};
use image::{Image, ImageDesc};
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

pub enum ResourceData {
    Image,
    Buffer,
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
        let node_resource = Node::Resource(NodeResource {
            id: 0,
            version: 0,
            name,
        });
        let node = self.framegraph.graph.add_node(node_resource);
        self.framegraph
            .graph
            .add_edge(self.pass_handle, node, "Create");
        self.framegraph.state.image_data.insert(node, desc);
        Resource::new(name, 0, node)
    }

    pub fn write<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let prev_resource = self.framegraph.graph[resource.handle]
            .to_resource()
            .expect("Should be a Resource");
        let node_resource = Node::Resource(NodeResource {
            id: resource.id,
            version: prev_resource.version + 1,
            name: resource.name,
        });
        let node = self.framegraph.graph.add_node(node_resource);
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, "Read");
        self.framegraph
            .graph
            .add_edge(self.pass_handle, node, "Write");
        Resource::new(resource.name, resource.id, node)
    }

    pub fn read<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, "Read");
        resource
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NodeResource {
    name: &'static str,
    id: usize,
    version: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct NodeRenderpass {
    name: &'static str,
}

#[derive(Debug, Copy, Clone)]
pub enum Node {
    Renderpass(NodeRenderpass),
    Resource(NodeResource),
}
impl Node {
    pub fn to_resource(self) -> Option<NodeResource> {
        match self {
            Node::Resource(r) => Some(r),
            _ => None,
        }
    }
    pub fn to_renderpass(self) -> Option<NodeRenderpass> {
        match self {
            Node::Renderpass(r) => Some(r),
            _ => None,
        }
    }
}
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
type TaskIndex = usize;
#[derive(Debug)]
struct TaskData {
    task_index: TaskIndex,
    inputs: Vec<TaskIndex>,
}
type ExecuteFn = Box<dyn Fn(&Context)>;
pub struct Compiled {
    images: HashMap<Handle, Image>,
    render: HashMap<Handle, Render>,
}

pub type ResourceMap = HashMap<Handle, Vec<Resource<Image>>>;
pub struct Recording {
    image_data: HashMap<Handle, ImageDesc>,
    image_resource_map: ResourceMap,
}

pub struct Framegraph<T = Recording> {
    blackboard: Blackboard,
    state: T,
    graph: Graph<Node, &'static str>,
    resources: Vec<()>,
    execute_fns: HashMap<Handle, Arc<dyn Execute>>,
}
pub trait GetResource<T> {
    fn get_resource(&self, resource: Resource<T>) -> &T;
}

impl GetResource<Image> for Framegraph<Compiled> {
    fn get_resource(&self, resource: Resource<Image>) -> &Image {
        self.state.images.get(&resource.handle).expect("get image")
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
                image_data: HashMap::new(),
                image_resource_map: HashMap::new(),
            },
            graph: Graph::new(),
            resources: Vec::new(),
            execute_fns: HashMap::new(),
            blackboard,
        }
    }
    pub fn add_render_pass<Data, Pass, Setup>(
        &mut self,
        name: &'static str,
        setup: Setup,
        pass: Pass,
        execute: fn(&Data, &Blackboard, &Render, &Framegraph<Compiled>),
    ) -> Arc<RenderTask<Data>>
    where
        Setup: Fn(&mut TaskBuilder) -> Data,
        Pass: Fn(&Data) -> Vec<Resource<Image>>,
        Data: 'static,
    {
        let (pass_handle, image_resources, task) = {
            let renderpass = NodeRenderpass { name };
            let pass_handle = self.graph.add_node(Node::Renderpass(renderpass));
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
    pub fn compile(self, ctx: &Context) -> Framegraph<Compiled> {
        let images: HashMap<_, _> = self
            .state
            .image_data
            .iter()
            .map(|(&node, image_desc)| {
                let image = Image::allocate(ctx, image_desc.clone());
                (node, image)
            })
            .collect();
        let render: HashMap<_, _> = self
            .state
            .image_resource_map
            .iter()
            .map(|(&handle, image_resources)| {
                let images: Vec<&Image> = image_resources
                    .iter()
                    .map(|&resource| images.get(&resource.handle).expect("resource"))
                    .collect();
                (handle, Render::new(ctx, &images))
            })
            .collect();
        let state = Compiled { images, render };
        Framegraph {
            execute_fns: self.execute_fns,
            resources: self.resources,
            graph: self.graph,
            state,
            blackboard: self.blackboard
        }
    }
}

impl Framegraph<Compiled> {
    pub fn execute(&self, ctx: &Context) {
        use petgraph::visit::{Bfs, Walker};
        let bfs = Bfs::new(&self.graph, Handle::new(0));
        bfs.iter(&self.graph)
            .filter(|&idx| match self.graph[idx] {
                Node::Renderpass(_) => true,
                _ => false,
            })
            .for_each(|idx| {
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
