extern crate ash;
extern crate tephra;
pub use tephra::winit;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use std::default::Default;
use std::marker::PhantomData;
use std::ptr;
use tephra::failure::Fail;

use tephra::backend::vulkan::{self, Context};
use tephra::backend::BackendApi;
use tephra::buffer::{Buffer, BufferUsage, Property};
use tephra::context;
use tephra::framegraph::*;
use tephra::image::{Image, ImageDesc, ImageLayout, RenderTarget, RenderTargetInfo, Resolution};
use tephra::pipeline::PipelineState;
use tephra::renderpass::{VertexInput, VertexInputData, VertexType};
use tephra::shader::Shader;
use tephra::swapchain::{Swapchain, SwapchainError};

#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

// TODO: Impl custom derive
impl VertexInput for Vertex {
    fn vertex_input_data() -> Vec<VertexInputData> {
        vec![
            VertexInputData {
                binding: 0,
                location: 0,
                offset: 0,
                vertex_type: VertexType::F32(4),
            },
            VertexInputData {
                binding: 0,
                location: 1,
                offset: 4 * 32,
                vertex_type: VertexType::F32(4),
            },
        ]
    }
}
pub struct TriangleData {
    pub color: Resource<Image>,
    pub depth: Resource<Image>,
}
pub fn triangle_pass(
    ctx: &context::Context,
    blackboard: Blackboard,
    resolution: Resolution,
) -> Framegraph<Compiled> {
    let mut fg = Framegraph::new(blackboard);
    let triangle_data = fg.add_render_pass(
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
        // TODO: Infer framebuffer layout based on data,
        |data| vec![data.color, data.depth],
        |data, blackboard, render, context| {
            // TODO: Find something better than a blackboard
            let r = blackboard.get::<TriangleState>().expect("State");
            render.draw_indexed(&r.state, &r.vertex_buffer, &r.index_buffer);
            let swapchain = blackboard.get::<Swapchain>().expect("swapchain");
            let color_image = context.get_resource(data.color);
            swapchain.copy_and_present(color_image);
        },
    );
    fg.compile(ctx)
}
struct TriangleState {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
    state: PipelineState,
}
fn main() {
    let context = Context::new();
    let mut swapchain = Swapchain::new(&context);
    let mut blackboard = Blackboard::new();
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
    let vertex_buffer = Buffer::from_slice(
        &context,
        Property::HostVisible,
        BufferUsage::Vertex,
        &vertices,
    ).expect("Failed to create vertex buffer");
    let index_buffer_data = [0u32, 1, 2];
    let index_buffer = Buffer::from_slice(
        &context,
        Property::HostVisible,
        BufferUsage::Index,
        &index_buffer_data,
    ).expect("index buffer");
    let vertex_shader_module = Shader::load(&context, "shader/triangle/vert.spv").expect("vertex");
    let fragment_shader_module =
        Shader::load(&context, "shader/triangle/frag.spv").expect("vertex");
    let state = PipelineState::new()
        .with_vertex_shader(vertex_shader_module)
        .with_fragment_shader(fragment_shader_module);
    let renderstate = TriangleState {
        vertex_buffer,
        index_buffer,
        state,
    };
    let resolution = swapchain.resolution();
    blackboard.add(renderstate);
    blackboard.add(swapchain);
    let fg = triangle_pass(&context, blackboard, resolution);
    loop {
        fg.execute(&context);
        std::thread::sleep_ms(2000);
    }
}
