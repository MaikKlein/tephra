use context::Context;
use image::Image;
use petgraph::{self, Graph};
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

pub enum ResourceData {
    Image,
    Buffer,
}

#[derive(Debug)]
pub struct Resource<T> {
    _m: PhantomData<T>,
    handle: Handle,
    id: usize,
    name: &'static str,
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
    pub fn create_image(&mut self, name: &'static str) -> Resource<Image> {
        let node_resource = Node::Resource(NodeResource {
            id: 0,
            version: 0,
            name,
        });
        let node = self.framegraph.graph.add_node(node_resource);
        self.framegraph
            .graph
            .add_edge(self.pass_handle, node, "Create");
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
pub struct Recording;

type TaskIndex = usize;
#[derive(Debug)]
struct TaskData {
    task_index: TaskIndex,
    inputs: Vec<TaskIndex>,
}
pub struct Compiled {}
pub struct Framegraph<T = Recording> {
    state: T,
    graph: Graph<Node, &'static str>,
    resources: Vec<()>,
    execute_fns: HashMap<Handle, Arc<dyn Execute>>,
}

pub struct RenderTask<T> {
    data: T,
    execute: fn(&T, &Context),
}

pub trait Execute {
    fn execute(&self, ctx: &Context);
}

impl<T> Execute for RenderTask<T> {
    fn execute(&self, ctx: &Context) {
        (self.execute)(&self.data, ctx)
    }
}
pub trait Task {
    type Data;
    fn execute(&self, ctx: &Context);
}
type ExecuteFn = Box<dyn Fn(&Context)>;
impl Framegraph {
    pub fn new() -> Self {
        Framegraph {
            state: Recording {},
            graph: Graph::new(),
            resources: Vec::new(),
            execute_fns: HashMap::new(),
        }
    }
    pub fn add_render_pass<Data, Setup>(
        &mut self,
        name: &'static str,
        setup: Setup,
        execute: fn(&Data, &Context),
    ) -> Arc<RenderTask<Data>>
    where
        Setup: Fn(&mut TaskBuilder) -> Data,
        Data: 'static,
    {
        let (pass_handle, task) = {
            let renderpass = NodeRenderpass { name };
            let pass_handle = self.graph.add_node(Node::Renderpass(renderpass));
            let mut builder = TaskBuilder {
                pass_handle,
                framegraph: self,
            };
            let data = setup(&mut builder);
            let task = RenderTask { data, execute };
            (pass_handle, Arc::new(task))
        };
        self.execute_fns.insert(pass_handle, task.clone());
        task
    }
    pub fn compile(self) -> Framegraph<Compiled> {
        Framegraph {
            execute_fns: self.execute_fns,
            resources: self.resources,
            graph: self.graph,
            state: Compiled {},
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
            }).for_each(|idx| {
                let execute = self.execute_fns.get(&idx).expect("renderpass");
                execute.execute(ctx);
            });
    }
    pub fn export_graphviz<P: AsRef<Path>>(&self, path: P) {
        use std::io::Write;
        let mut file = File::create(path.as_ref()).expect("path");
        let dot = petgraph::dot::Dot::with_config(&self.graph, &[]);
        write!(&mut file, "{}", dot);
    }
}
