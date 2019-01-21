extern crate ash;
extern crate tephra;
extern crate tephra_vulkan;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use tephra::{
    buffer::{Buffer, BufferUsage, Property},
    commandbuffer::{
        CommandList, Compute, Descriptor, Graphics, ShaderArguments, ShaderResource, ShaderView,
    },
    context::Context,
    descriptor::DescriptorType,
    framegraph::{Blackboard, Framegraph, ReadResource, Recording, Resource, WriteResource},
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
    pub buffer: Buffer<[f32; 4]>,
}

pub struct TriangleCompute {
    pub storage_buffer: WriteResource<Buffer<[f32; 4]>>,
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
            (pass, move |registry, _, cmds| {
                let descriptor = Descriptor::builder()
                    .with(
                        registry.get_buffer(storage_buffer),
                        0,
                        DescriptorType::Storage,
                    )
                    .build();
                let args = ShaderArguments::builder()
                    .with_shader_arg(0, descriptor)
                    .build();
                cmds.record::<Compute>(registry)
                    .dispatch(pipeline, args, 1, 1, 1)
                    .submit();
            })
        })
    }
}

#[derive(Descriptor)]
pub struct Color {
    #[descriptor(Storage)]
    pub color: Buffer<[f32; 4]>,
}
#[derive(Copy, Clone)]
pub struct TrianglePass {
    pub storage_buffer: ReadResource<Buffer<[f32; 4]>>,
    pub color: WriteResource<Image>,
    pub depth: WriteResource<Image>,
}

impl TrianglePass {
    pub unsafe fn add_pass(
        ctx: &Context,
        fg: &mut Framegraph<Recording>,
        storage_buffer: impl Resource<Type = Buffer<[f32; 4]>>,
        resolution: Resolution,
        format: Format,
    ) -> TrianglePass {
        fg.add_pass("Triangle Pass", |builder| {
            let storage_buffer = builder.read(storage_buffer);
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
            (pass, move |registry, blackbox, cmds| {
                let state = blackbox.get::<TriangleState>().expect("State");
                let shader_arguments = Descriptor::builder()
                    .with(
                        registry.get_buffer(pass.storage_buffer),
                        0,
                        DescriptorType::Storage,
                    )
                    .build();
                let space = ShaderArguments::builder()
                    .with_shader_arg(0, shader_arguments)
                    .build();
                cmds.record::<Graphics>(registry)
                    .draw_indexed(
                        pipeline,
                        renderpass,
                        framebuffer,
                        space,
                        state.vertex_buffer,
                        state.index_buffer,
                        0..3,
                    )
                    .submit();
            })
        })
    }
}

#[derive(Copy, Clone)]
pub struct Presentpass {
    pub color: ReadResource<Image>,
}

impl Presentpass {
    pub fn add_pass(fg: &mut Framegraph<Recording>, color: impl Resource<Type = Image>) {
        fg.add_pass("PresentPass", |builder| {
            let pass = Presentpass {
                color: builder.read(color),
            };
            (pass, move |registry, blackboard, _cmds| {
                let swapchain = blackboard.get::<Swapchain>().expect("swap");
                let color_image = registry.get_image(pass.color);
                swapchain.copy_and_present(color_image);
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
