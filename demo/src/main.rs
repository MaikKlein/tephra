extern crate ash;
extern crate tephra;
extern crate tephra_vulkan;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use tephra::{
    buffer::{Buffer, BufferUsage, Property},
    commandbuffer::{CommandList, Compute, Graphics},
    context::Context,
    framegraph::{Blackboard, Framegraph, Recording, Resource},
    image::{Format, Image, ImageDesc, ImageLayout, Resolution},
    pipeline::{ComputePipeline, GraphicsPipeline, ShaderStage},
    renderpass::{Attachment, Renderpass},
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
}

impl TriangleCompute {
    pub unsafe fn add_pass(ctx: &Context, fg: &mut Framegraph<Recording>) -> TriangleCompute {
        fg.add_pass("Compute", |builder| {
            let storage_buffer = {
                let buffer = Buffer::from_slice(
                    ctx,
                    Property::HostVisible,
                    BufferUsage::Storage,
                    &[[1.0f32, 0.0, 0.0, 1.0]],
                )
                .expect("Buffer");
                builder.add_buffer(buffer)
            };

            let compute_shader =
                ShaderModule::load(ctx, "shader/triangle/comp.spv").expect("compute shader");
            let pipeline = ComputePipeline::builder()
                .compute_shader(ShaderStage {
                    shader_module: compute_shader,
                    entry_name: "main".into(),
                })
                .layout::<Color>()
                .create(ctx);
            let pass = TriangleCompute { storage_buffer };
            (pass, move |fg, _, pool| {
                let mut cmds = CommandList::new();
                let color = ComputeDesc {
                    buffer: storage_buffer,
                };
                let mut descriptor = pool.allocate::<ComputeDesc>();
                descriptor.update(fg.ctx(), &color, &fg);
                cmds.record::<Compute>()
                    .dispatch(pipeline, descriptor, 1, 1, 1)
                    .submit();
                cmds
            })
        })
    }
}

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
    pub unsafe fn add_pass(
        ctx: &Context,
        fg: &mut Framegraph<Recording>,
        storage_buffer: Resource<Buffer<[f32; 4]>>,
        resolution: Resolution,
        format: Format,
    ) -> TrianglePass {
        fg.add_pass("Triangle Pass", |builder| {
            let vertex_shader_module =
                ShaderModule::load(&ctx, "shader/triangle/vert.spv").expect("vertex");
            let fragment_shader_module =
                ShaderModule::load(&ctx, "shader/triangle/frag.spv").expect("vertex");
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
            let renderpass = Renderpass::builder()
                .color_attachment(
                    Attachment::builder()
                        .format(format)
                        .index(0)
                        .build()
                        .unwrap(),
                )
                .with_depth_attachment(
                    Attachment::builder()
                        .format(Format::D16_UNORM)
                        .index(1)
                        .build()
                        .unwrap(),
                )
                .create(ctx);
            let pipeline = GraphicsPipeline::builder()
                .vertex_shader(ShaderStage {
                    shader_module: vertex_shader_module,
                    entry_name: "main".into(),
                })
                .fragment_shader(ShaderStage {
                    shader_module: fragment_shader_module,
                    entry_name: "main".into(),
                })
                .render_target(renderpass)
                .layout::<Color>()
                .vertex::<Vertex>()
                .create(ctx);
            let framebuffer = builder.create_framebuffer(renderpass, vec![pass.color, pass.depth]);
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
                        pipeline,
                        renderpass,
                        fg.registry().get_framebuffer(framebuffer),
                        descriptor,
                        state.vertex_buffer,
                        state.index_buffer,
                        0..3,
                    )
                    .submit();
                cmds
            })
        })
    }
}

#[derive(Copy, Clone)]
pub struct Presentpass {
    pub color: Resource<Image>,
}

impl Presentpass {
    pub fn add_pass(fg: &mut Framegraph<Recording>, color: Resource<Image>) {
        fg.add_pass("PresentPass", |builder| {
            let pass = Presentpass {
                color: builder.read(color),
            };
            (pass, move |fg, blackboard, _pool| {
                let swapchain = blackboard.get::<Swapchain>().expect("swap");
                let color_image = fg.registry().get_image(pass.color);
                swapchain.copy_and_present(color_image);
                let cmds = CommandList::new();
                cmds
            })
        });
    }
}

pub unsafe fn render_pass(
    ctx: &Context,
    fg: &mut Framegraph<Recording>,
    resolution: Resolution,
    swapchain: &Swapchain,
) {
    let triangle_compute = TriangleCompute::add_pass(ctx, fg);
    let triangle_data = TrianglePass::add_pass(
        ctx,
        fg,
        triangle_compute.storage_buffer,
        resolution,
        swapchain.format(),
    );
    Presentpass::add_pass(fg, triangle_data.color);
}

struct TriangleState {
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
}
fn main() {
    unsafe {
        let ctx = tephra_vulkan::Context::new();
        let mut blackboard = Blackboard::new();
        let swapchain = Swapchain::new(&ctx);
        let resolution = swapchain.resolution();
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
        render_pass(&ctx, &mut fg, resolution, &swapchain);
        blackboard.add(swapchain);
        let mut fg = fg.compile();
        fg.export_graphviz("graph.dot");
        loop {
            // Execute the graph every frame
            fg.execute(&blackboard);
        }
    }
}
