extern crate ash;
extern crate tephra;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use std::sync::Arc;
use tephra::{
    backend::vulkan::Context,
    buffer::{Buffer, BufferUsage, Property},
    commandbuffer::{ComputeCommandbuffer, GraphicsCommandbuffer},
    framegraph::{
        render_task::{Computepass, Renderpass},
        Blackboard, Compiled, Framegraph, GetResource, Recording, Resource,
    },
    image::{Image, ImageDesc, ImageLayout, Resolution},
    pipeline::{ComputeState, PipelineState},
    shader::ShaderModule,
    swapchain::Swapchain,
};

#[derive(Clone, Debug, Copy)]
#[repr(C)]
#[derive(VertexInput)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Descriptor)]
pub struct ComputeDesc {
    #[descriptor(Storage)]
    pub buffer: Resource<Buffer<[f32; 4]>>,
}

pub struct TriangleCompute {
    pub storage_buffer: Resource<Buffer<[f32; 4]>>,
    pub state: ComputeState,
}

impl TriangleCompute {
    pub fn add_pass(fg: &mut Framegraph<Recording>) -> Arc<TriangleCompute> {
        let buffer = Buffer::from_slice(
            &fg.ctx,
            Property::HostVisible,
            BufferUsage::Storage,
            &[[1.0f32, 0.0, 0.0, 1.0]],
        ).expect("Buffer");
        let compute_shader =
            ShaderModule::load(&fg.ctx, "shader/triangle/comp.spv").expect("compute shader");
        let storage_buffer = fg.add_buffer(buffer);
        fg.add_compute_pass("Compute", move |builder| TriangleCompute {
            storage_buffer: builder.write(storage_buffer),
            state: ComputeState {
                compute_shader: Some(compute_shader.clone()),
            },
        })
    }
}

impl Computepass for TriangleCompute {
    type Layout = ComputeDesc;
    fn execute<'cmd>(
        &'cmd self,
        _blackboard: &'cmd Blackboard,
        cmds: &mut ComputeCommandbuffer<'cmd>,
        _fg: &Framegraph<Compiled>,
    ) {
        let desc = ComputeDesc {
            buffer: self.storage_buffer,
        };
        cmds.bind_pipeline(&self.state);
        cmds.bind_descriptor(&desc);
        cmds.dispatch(1, 1, 1);
    }
}

#[derive(Descriptor)]
pub struct Color {
    #[descriptor(Storage)]
    pub color: Resource<Buffer<[f32; 4]>>,
}
pub struct TrianglePass {
    pub storage_buffer: Resource<Buffer<[f32; 4]>>,
    pub color: Resource<Image>,
    pub depth: Resource<Image>,
}

impl Renderpass for TrianglePass {
    type Vertex = Vertex;
    type Layout = Color;
    fn framebuffer(&self) -> Vec<Resource<Image>> {
        vec![self.color, self.depth]
    }
    fn execute<'a>(
        &'a self,
        blackboard: &'a Blackboard,
        cmds: &mut GraphicsCommandbuffer<'a>,
        _fg: &Framegraph<Compiled>,
    ) {
        let color = Color {
            color: self.storage_buffer,
        };
        let r = blackboard.get::<TriangleState>().expect("state");
        let shader = blackboard.get::<TriangleShader>().expect("shader");
        shader.draw_index(&r.vertex_buffer, &r.index_buffer, &r.state, &color, cmds);
    }
}

impl TrianglePass {
    pub fn add_pass(
        fg: &mut Framegraph<Recording>,
        storage_buffer: Resource<Buffer<[f32; 4]>>,
        resolution: Resolution,
    ) -> Arc<TrianglePass> {
        fg.add_render_pass("Triangle Pass", |builder| {
            let color_desc = ImageDesc {
                layout: ImageLayout::Color,
                resolution,
            };
            let depth_desc = ImageDesc {
                layout: ImageLayout::Depth,
                resolution,
            };
            TrianglePass {
                color: builder.create_image("Color", color_desc),
                depth: builder.create_image("Depth", depth_desc),
                storage_buffer: builder.read(storage_buffer),
            }
        })
    }
}

pub struct Presentpass {
    pub color: Resource<Image>,
}

impl Computepass for Presentpass {
    type Layout = ();
    fn execute<'cmd>(
        &'cmd self,
        blackboard: &'cmd Blackboard,
        _cmds: &mut ComputeCommandbuffer<'cmd>,
        fg: &Framegraph<Compiled>,
    ) {
        let swapchain = blackboard.get::<Swapchain>().expect("swap");
        let color_image = fg.get_resource(self.color);
        swapchain.copy_and_present(color_image);
    }
}

impl Presentpass {
    pub fn add_pass(fg: &mut Framegraph<Recording>, color: Resource<Image>) {
        fg.add_compute_pass("PresentPass", |builder| Presentpass {
            color: builder.read(color),
        });
    }
}

pub fn render_pass(fg: &mut Framegraph<Recording>, resolution: Resolution) {
    let triangle_compute = TriangleCompute::add_pass(fg);
    let triangle_data = TrianglePass::add_pass(fg, triangle_compute.storage_buffer, resolution);
    Presentpass::add_pass(fg, triangle_data.color);
}

struct TriangleState {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
    state: PipelineState,
}
pub struct TriangleShader {}
impl TriangleShader {
    pub fn new() -> Self {
        TriangleShader {}
    }

    pub fn draw_index<'a>(
        &'a self,
        vertex_buffer: &'a Buffer<Vertex>,
        index_buffer: &'a Buffer<u32>,
        state: &'a PipelineState,
        color: &Color,
        cmds: &mut GraphicsCommandbuffer<'a>,
    ) {
        cmds.bind_vertex(vertex_buffer);
        cmds.bind_index(index_buffer);
        cmds.bind_pipeline::<Vertex>(state);
        cmds.bind_descriptor(color);
        cmds.draw_index(3);
    }
}
fn main() {
    let ctx = Context::new();
    let mut blackboard = Blackboard::new();
    let swapchain = Swapchain::new(&ctx);
    let resolution = swapchain.resolution();
    let vertex_shader_module =
        ShaderModule::load(&ctx, "shader/triangle/vert.spv").expect("vertex");
    let fragment_shader_module =
        ShaderModule::load(&ctx, "shader/triangle/frag.spv").expect("vertex");
    let state = PipelineState::new()
        .with_vertex_shader(vertex_shader_module)
        .with_fragment_shader(fragment_shader_module);
    let index_buffer_data = [0u32, 1, 2];
    let index_buffer = Buffer::from_slice(
        &ctx,
        Property::HostVisible,
        BufferUsage::Index,
        &index_buffer_data,
    ).expect("index buffer");
    let vertices = [
        Vertex {
            pos: [-1.0, 1.0, 0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0, 0.0, 1.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0, 0.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
    ];

    let vertex_buffer =
        Buffer::from_slice(&ctx, Property::HostVisible, BufferUsage::Vertex, &vertices)
            .expect("Failed to create vertex buffer");

    let triangle_shader = TriangleShader::new();
    blackboard.add(triangle_shader);
    blackboard.add(swapchain);
    let mut fg = Framegraph::new(&ctx);
    let triangle_state = TriangleState {
        vertex_buffer,
        index_buffer,
        state,
    };
    blackboard.add(triangle_state);
    render_pass(&mut fg, resolution);
    let mut fg = fg.compile(resolution, &ctx);
    fg.export_graphviz("graph.dot");
    loop {
        // Execute the graph every frame
        fg.execute(&blackboard);
    }
}
