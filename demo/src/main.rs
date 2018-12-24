extern crate ash;
extern crate tephra;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use std::sync::Arc;
use tephra::{
    backend::vulkan::Context,
    buffer::{Buffer, BufferUsage, Property},
    commandbuffer::{CommandList, Compute, Graphics, Transfer},
    framegraph::{
        task_builder::deferred::Attachment, Blackboard, Compiled, Framegraph, GetResource,
        Recording, Resource,
    },
    image::{Format, Image, ImageDesc, ImageLayout, Resolution},
    pipeline::{ComputeState, GraphicsPipeline, PipelineState, ShaderStage},
    renderpass::RenderTarget,
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

// pub struct TriangleCompute {
//     pub storage_buffer: Resource<Buffer<[f32; 4]>>,
//     pub state: ComputeState,
// }

// impl TriangleCompute {
//     pub fn add_pass(fg: &mut Framegraph<Recording>) -> Arc<TriangleCompute> {
//         let buffer = Buffer::from_slice(
//             &fg.ctx,
//             Property::HostVisible,
//             BufferUsage::Storage,
//             &[[1.0f32, 0.0, 0.0, 1.0]],
//         )
//         .expect("Buffer");
//         let compute_shader =
//             ShaderModule::load(&fg.ctx, "shader/triangle/comp.spv").expect("compute shader");
//         let storage_buffer = fg.add_buffer(buffer);
//         fg.add_compute_pass("Compute", move |builder| {
//             TriangleCompute {
//                 storage_buffer: builder.write(storage_buffer),
//                 state: ComputeState {
//                     compute_shader: Some(compute_shader.clone()),
//                 },
//             }
//         })
//     }
// }

// impl Computepass for TriangleCompute {
//     type Layout = ComputeDesc;
//     fn execute<'cmd>(
//         &'cmd self,
//         _blackboard: &'cmd Blackboard,
//         cmds: &mut ComputeCommandbuffer<'cmd>,
//         _fg: &Framegraph<Compiled>,
//     ) {
//         let desc = ComputeDesc {
//             buffer: self.storage_buffer,
//         };
//         cmds.bind_pipeline(&self.state);
//         cmds.bind_descriptor(&desc);
//         cmds.dispatch(1, 1, 1);
//     }
// }

#[derive(Descriptor)]
pub struct Color {
    #[descriptor(Storage)]
    pub color: Resource<Buffer<[f32; 4]>>,
}
#[derive(Copy, Clone)]
pub struct TrianglePass {
    pub storage_buffer: Resource<Buffer<[f32; 4]>>,
    pub color: Resource<Image>,
    pub depth: Resource<Image>,
}

impl TrianglePass {
    pub fn add_pass(
        fg: &mut Framegraph<Recording>,
        storage_buffer: Resource<Buffer<[f32; 4]>>,
        resolution: Resolution,
        format: Format,
    ) -> TrianglePass {
        fg.add_pass("Triangle Pass", |builder| {
            let color_desc = ImageDesc {
                layout: ImageLayout::Color,
                format,
                resolution,
            };
            let depth_desc = ImageDesc {
                layout: ImageLayout::Depth,
                format: Format::D16_UNORM,
                resolution,
            };

            let pass = TrianglePass {
                color: builder.create_image("Color", color_desc),
                depth: builder.create_image("Depth", depth_desc),
                storage_buffer: builder.read(storage_buffer),
            };
            let render_target = RenderTarget::deferred()
                .color_attachment(
                    Attachment::builder()
                        .image(pass.color)
                        .index(0)
                        .build()
                        .unwrap(),
                )
                .with_depth_attachment(
                    Attachment::builder()
                        .image(pass.depth)
                        .index(1)
                        .build()
                        .unwrap(),
                )
                .build_deferred(builder);
            let pipeline = GraphicsPipeline::deferred()
                .render_target(render_target)
                .layout::<Color>()
                .build_deferred(builder);
            (pass, move |fg, blackbox, pool| {
                let mut cmds = CommandList::new();
                let state = blackbox.get::<TriangleState>().expect("State");
                let color = Color {
                    color: pass.storage_buffer,
                };
                let mut descriptor = pool.allocate::<Color>();
                // TODO: Improve this API, just terrible.
                descriptor.update(fg.ctx(), &color, &fg);
                cmds.record::<Graphics>()
                    .draw_indexed(
                        fg.registry().get_graphics_pipeline(pipeline),
                        descriptor,
                        state.vertex_buffer,
                        state.index_buffer,
                    )
                    .submit();
                cmds
            })
        })
    }
}

// pub struct Presentpass {
//     pub color: Resource<Image>,
// }

// impl Computepass for Presentpass {
//     type Layout = ();
//     fn execute<'cmd>(
//         &'cmd self,
//         blackboard: &'cmd Blackboard,
//         _cmds: &mut ComputeCommandbuffer<'cmd>,
//         fg: &Framegraph<Compiled>,
//     ) {
//         let swapchain = blackboard.get::<Swapchain>().expect("swap");
//         let color_image = fg.get_resource(self.color);
//         swapchain.copy_and_present(color_image);
//     }
// }

// impl Presentpass {
//     pub fn add_pass(
//         fg: &mut Framegraph<Recording>,
//         color: Resource<Image>,
//     ) {
//         fg.add_compute_pass("PresentPass", |builder| {
//             Presentpass {
//                 color: builder.read(color),
//             }
//         });
//     }
// }

// pub fn render_pass(
//     fg: &mut Framegraph<Recording>,
//     resolution: Resolution,
//     swapchain: &Swapchain,
// ) {
//     let triangle_compute = TriangleCompute::add_pass(fg);
//     let triangle_data = TrianglePass::add_pass(
//         fg,
//         triangle_compute.storage_buffer,
//         resolution,
//         swapchain.format(),
//     );
//     //Presentpass::add_pass(fg, triangle_data.color);
// }

struct TriangleState {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
}
// pub struct TriangleShader {}
// impl TriangleShader {
//     pub fn new() -> Self {
//         TriangleShader {}
//     }

//     pub fn draw_index<'a>(
//         &'a self,
//         vertex_buffer: Buffer<Vertex>,
//         index_buffer: Buffer<u32>,
//         state: &'a PipelineState,
//         color: &Color,
//         cmds: &mut GraphicsCommandbuffer<'a>,
//     ) {
//         cmds.bind_vertex(vertex_buffer);
//         cmds.bind_index(index_buffer);
//         cmds.bind_pipeline::<Vertex>(state);
//         cmds.bind_descriptor(color);
//         cmds.draw_index(3);
//     }
// }
fn main() {
    unsafe {
        let ctx = Context::new();
        let mut blackboard = Blackboard::new();
        let swapchain = Swapchain::new(&ctx);
        let resolution = swapchain.resolution();
        let vertex_shader_module =
            ShaderModule::load(&ctx, "shader/triangle/vert.spv").expect("vertex");
        let fragment_shader_module =
            ShaderModule::load(&ctx, "shader/triangle/frag.spv").expect("vertex");
        // let state = PipelineState::new()
        //     .with_vertex_shader(ShaderStage {
        //         shader_module: vertex_shader_module,
        //         entry_name: "main".into(),
        //     })
        //     .with_fragment_shader(ShaderStage {
        //         shader_module: fragment_shader_module,
        //         entry_name: "main".into(),
        //     });
        let index_buffer_data = [0u32, 1, 2];
        let index_buffer = Buffer::from_slice(
            &ctx,
            Property::HostVisible,
            BufferUsage::Index,
            &index_buffer_data,
        )
        .expect("index buffer");
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

        let mut fg = Framegraph::new(&ctx);
        let triangle_state = TriangleState {
            vertex_buffer,
            index_buffer,
        };
        blackboard.add(triangle_state);
        //render_pass(&mut fg, resolution, &swapchain);
        blackboard.add(swapchain);
        let mut fg = fg.compile();
        fg.export_graphviz("graph.dot");
        loop {
            // Execute the graph every frame
            fg.execute(&blackboard);
        }
    }
}
