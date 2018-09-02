extern crate ash;
extern crate tephra;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use tephra::backend::vulkan::Context;
use tephra::buffer::{Buffer, BufferUsage, GenericBuffer, Property};
use tephra::context;
use tephra::framegraph::render_task::ARenderTask;
use tephra::framegraph::{Blackboard, Compiled, Framegraph, GetResource, Recording, Resource};
use tephra::image::{Image, ImageDesc, ImageLayout, Resolution};
use tephra::pipeline::PipelineState;
use tephra::shader::ShaderModule;
use tephra::swapchain::Swapchain;

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
pub fn add_triangle_pass(
    fg: &mut Framegraph<Recording>,
    resolution: Resolution,
) -> ARenderTask<TriangleData> {
    fg.add_render_pass(
        "Triangle Pass",
        |builder| {
            let color_desc = ImageDesc {
                layout: ImageLayout::Color,
                resolution,
            };
            let depth_desc = ImageDesc {
                layout: ImageLayout::Depth,
                resolution,
            };
            TriangleData {
                color: builder.create_image("Color", color_desc),
                depth: builder.create_image("Depth", depth_desc),
            }
        },
        // TODO: Infer framebuffer layout based on data/shader,
        |data| vec![data.color, data.depth],
        |data, blackboard, cmds, context| {
            {
                let r = blackboard.get::<TriangleState>().expect("state");
                // render.draw_indexed(&r.state, &r.vertex_buffer, &r.index_buffer, &r.descriptors);
                cmds.bind_vertex(&r.vertex_buffer);
                cmds.bind_index(&r.index_buffer);
                // TODO: terrible, don't clone
                cmds.bind_pipeline::<Vertex>(r.state.clone());
                cmds.draw_index(3);
            }
            let swapchain = blackboard.get::<Swapchain>().expect("swap");
            let color_image = context.get_resource(data.color);
            swapchain.copy_and_present(color_image);
        },
    )
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
    let mut fg = Framegraph::new();
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
    descriptors: Vec<u32>,
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

    let triangle_state = TriangleState {
        vertex_buffer,
        index_buffer,
        state,
        descriptors: vec![1, 2, 3],
    };
    blackboard.add(triangle_state);
    blackboard.add(swapchain);
    let render_pass = render_pass(&ctx, resolution);
    loop {
        // Execute the graph every frame
        render_pass.execute(&blackboard);
    }
}
