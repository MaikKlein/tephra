extern crate ash;
extern crate tephra;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use std::sync::Arc;
use tephra::backend::vulkan::Context;
use tephra::buffer::{Buffer, BufferUsage, GenericBuffer, Property};
use tephra::commandbuffer::GraphicsCommandbuffer;
use tephra::context;
use tephra::descriptor::{
    Allocator, Binding, Descriptor, DescriptorInfo, DescriptorResource, DescriptorSizes,
    DescriptorType, Pool,
};
use tephra::framegraph::render_task::Renderpass;
use tephra::framegraph::{Blackboard, Compiled, Framegraph, GetResource, Recording, Resource};
use tephra::image::{Image, ImageDesc, ImageLayout, Resolution};
use tephra::pipeline::PipelineState;
use tephra::renderpass::VertexInput;
use tephra::shader::ShaderModule;
use tephra::swapchain::Swapchain;

pub struct Color {
    pub color: Buffer<[f32; 4]>,
}
impl DescriptorInfo for Color {
    fn descriptor_data(&self) -> Vec<Binding<DescriptorResource>> {
        vec![Binding {
            binding: 0,
            data: DescriptorResource::Uniform(&self.color.buffer),
        }]
    }
    fn sizes() -> DescriptorSizes {
        DescriptorSizes {
            buffer: 1,
            images: 0,
        }
    }
    fn layout() -> Vec<Binding<DescriptorType>> {
        vec![Binding {
            binding: 0,
            data: DescriptorType::Uniform,
        }]
    }
}

#[derive(Clone, Debug, Copy)]
#[repr(C)]
#[derive(VertexInput)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

pub struct TriangleData {
    pub color: Resource<Image>,
    pub depth: Resource<Image>,
}
pub struct TrianglePass {
    pub color: Resource<Image>,
    pub depth: Resource<Image>,
}

impl<'graph> Renderpass<'graph> for TrianglePass {
    type Vertex = Vertex;
    type Layout = Color;
    fn framebuffer(&self) -> Vec<Resource<Image>> {
        vec![self.color, self.depth]
    }
    fn execute<'a>(
        &self,
        blackboard: &'a Blackboard,
        cmds: &mut GraphicsCommandbuffer<'a>,
        fg: &Framegraph<'graph, Compiled>,
    ) {
        {
            let r = blackboard.get::<TriangleState>().expect("state");
            let shader = blackboard.get::<TriangleShader>().expect("shader");
            shader.draw_index(&r.vertex_buffer, &r.index_buffer, &r.state, &r.color, cmds);
        }
        let swapchain = blackboard.get::<Swapchain>().expect("swap");
        let color_image = fg.get_resource(self.color);
        swapchain.copy_and_present(color_image);
    }
}

pub fn add_triangle_pass<'graph>(
    fg: &mut Framegraph<'graph, Recording>,
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
        }
    })
}

// pub fn add_present_pass(fg: &mut Framegraph<Recording>, color: Resource<Image>) {
//     struct PresentData {
//         color: Resource<Image>,
//     }
//     fg.add_render_pass(
//         "Present Pass",
//         |builder| PresentData {
//             color: builder.read(color),
//         },
//         |_data| vec![],
//         |data, blackboard, _render, context| {
//             let swapchain = blackboard.get::<Swapchain>().expect("swap");
//             let color_image = context.get_resource(data.color);
//             swapchain.copy_and_present(color_image);
//         },
//     );
// }

pub fn render_pass(ctx: &context::Context, resolution: Resolution) -> Framegraph<Compiled> {
    let mut fg = Framegraph::new(ctx);
    let _triangle_data = add_triangle_pass(&mut fg, resolution);
    //add_present_pass(&mut fg, triangle_data.color);
    // Compiles the graph, allocates and optimizes resources
    fg.compile(resolution, ctx)
}
// Just state for the triangle pass
struct TriangleState {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
    state: PipelineState,
    color: Color,
}
use std::ops::Range;
pub trait GraphicsShader {
    type VertexInput: VertexInput;
    type Descriptor: DescriptorInfo;
    fn draw_indexed<'cmd>(
        &self,
        vertex_buffer: &'cmd Buffer<Self::VertexInput>,
        index_buffer: &'cmd Buffer<u32>,
        state: &'cmd PipelineState,
        range: Range<usize>,
        descriptors: &'cmd [Self::Descriptor],
        cmds: &mut GraphicsCommandbuffer<'cmd>,
    ) {
        cmds.bind_vertex(vertex_buffer);
        cmds.bind_index(index_buffer);
        cmds.bind_pipeline::<Self::VertexInput>(state);
        for desc in descriptors {
            cmds.bind_descriptor(desc);
            cmds.draw_index(range.end);
        }
    }
}

// pub struct Shader<S: GraphicsShader> {
//     vertex_shader: ShaderModule,
//     fragment_shader: ShaderModule,
// }

// use std::path::Path;
// impl<S: GraphicsShader> Shader<S> {
//     pub fn new(
//         ctx: &tephra::context::Context,
//         vertex_shader: ShaderModule,
//         fragment_shader: ShaderModule,
//     ) -> Self {
//         Shader {
//             vertex_shader,
//             fragment_shader,
//         }
//     }
// }

pub struct TriangleShader {}

// impl GraphicsShader for TriangleShader {
//     type VertexInput = Vertex;
//     type Descriptor = Color;
// }

impl TriangleShader {
    pub fn new(ctx: &tephra::context::Context) -> Self {
        TriangleShader {}
    }

    pub fn draw_index<'a>(
        &'a self,
        vertex_buffer: &'a Buffer<Vertex>,
        index_buffer: &'a Buffer<u32>,
        state: &'a PipelineState,
        color: &'a Color,
        cmds: &mut GraphicsCommandbuffer<'a>,
    ) {
        cmds.bind_vertex(vertex_buffer);
        cmds.bind_index(index_buffer);
        // TODO: terrible, don't clone
        cmds.bind_pipeline::<Vertex>(state);
        cmds.bind_descriptor(color);
        cmds.draw_index(3);
    }
}
fn main() {
    let ctx = Context::new();
    let color_buffer = Buffer::from_slice(
        &ctx,
        Property::HostVisible,
        BufferUsage::Uniform,
        &[[1.0f32, 0.0, 0.0, 1.0]],
    ).expect("color buffer");
    let color = Color {
        color: color_buffer,
    };
    // let pool = Pool::<Color>::new(&ctx);
    // let mut color_allocator = pool.allocate();

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

    let triangle_state = TriangleState {
        vertex_buffer,
        index_buffer,
        state,
        color,
    };
    let triangle_shader = TriangleShader::new(&ctx);
    blackboard.add(triangle_shader);
    blackboard.add(triangle_state);
    blackboard.add(swapchain);
    let render_pass = render_pass(&ctx, resolution);
    loop {
        // Execute the graph every frame
        render_pass.execute(&blackboard);
    }
}
