extern crate ash;
extern crate tephra;
extern crate tephra_vulkan;
#[macro_use]
extern crate tephra_derive;
pub use tephra::winit;

use tephra::{
    buffer::{Buffer, BufferUsage, Property},
    commandbuffer::{Access, CommandList, Compute, Descriptor, Graphics, ShaderArguments},
    context::Context,
    descriptor::{DescriptorType, Pool},
    image::{Format, Image, ImageDesc, ImageLayout},
    pipeline::{ComputePipeline, GraphicsPipeline, ShaderStage},
    renderpass::{Attachment, Framebuffer, Renderpass},
    shader::ShaderModule,
    swapchain::Swapchain,
    Error,
};

#[derive(Clone, Debug, Copy)]
#[repr(C)]
#[derive(VertexInput)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Descriptor)]
pub struct Color {
    #[descriptor(Storage)]
    pub color: Buffer<[f32; 4]>,
}

#[derive(Descriptor)]
pub struct ComputeDesc {
    #[descriptor(Storage)]
    pub buffer: Buffer<[f32; 4]>,
}

const VERTICES: [Vertex; 3] = [
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

pub struct Triangle {
    storage_buffer: Buffer<[f32; 4]>,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
    swapchain: Swapchain,
    graphics_pipeline: GraphicsPipeline,
    compute_pipeline: ComputePipeline,
    framebuffer: Framebuffer,
    renderpass: Renderpass,
    color_image: Image,
    depth_image: Image,
}

impl Triangle {
    pub unsafe fn new(ctx: &Context) -> Result<Triangle, Error> {
        let swapchain = Swapchain::new(&ctx);
        let resolution = swapchain.resolution();
        let index_buffer_data = [0u32, 1, 2];
        let index_buffer = Buffer::from_slice(
            &ctx,
            Property::HostVisible,
            BufferUsage::Index,
            &index_buffer_data,
        )?;
        let storage_buffer = Buffer::from_slice(
            ctx,
            Property::HostVisible,
            BufferUsage::Storage,
            &[[1.0f32, 0.0, 0.0, 1.0]],
        )?;

        let vertex_buffer =
            Buffer::from_slice(&ctx, Property::HostVisible, BufferUsage::Vertex, &VERTICES)?;

        let compute_pipeline = {
            let compute_shader = ShaderModule::load(ctx, "shader/triangle/comp.spv")?;
            ComputePipeline::builder()
                .compute_shader(ShaderStage {
                    shader_module: compute_shader,
                    entry_name: "main".into(),
                })
                .layout::<Color>()
                .create(ctx)
        };
        let format = swapchain.format();
        let vertex_shader_module = ShaderModule::load(&ctx, "shader/triangle/vert.spv")?;
        let fragment_shader_module = ShaderModule::load(&ctx, "shader/triangle/frag.spv")?;

        let color_desc = ImageDesc {
            layout: ImageLayout::Color,
            format,
            resolution,
        };
        let color = Image::allocate(ctx, color_desc);

        let depth_desc = ImageDesc {
            layout: ImageLayout::Depth,
            format: Format::D16_UNORM,
            resolution,
        };
        let depth = Image::allocate(ctx, depth_desc);

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

        let graphics_pipeline = GraphicsPipeline::builder()
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
        let framebuffer = ctx.create_framebuffer(renderpass, &[color, depth]);
        let triangle = Triangle {
            vertex_buffer,
            storage_buffer,
            index_buffer,
            swapchain,
            compute_pipeline,
            graphics_pipeline,
            framebuffer,
            renderpass,
            color_image: color,
            depth_image: depth,
        };
        Ok(triangle)
    }

    pub fn record_commands(&self, cmds: &mut CommandList) {
        let descriptor = Descriptor::builder()
            .with(
                self.storage_buffer,
                0,
                DescriptorType::Storage,
                Access::Write,
            )
            .build();
        let args = ShaderArguments::builder()
            .with_shader_arg(0, descriptor)
            .build();
        cmds.record::<Compute>()
            .dispatch(self.compute_pipeline, args, 1, 1, 1)
            .submit();
        let shader_arguments = Descriptor::builder()
            .with(
                self.storage_buffer,
                0,
                DescriptorType::Storage,
                Access::Read,
            )
            .build();
        let space = ShaderArguments::builder()
            .with_shader_arg(0, shader_arguments)
            .build();
        cmds.record::<Graphics>()
            .draw_indexed(
                self.graphics_pipeline,
                self.renderpass,
                self.framebuffer,
                space,
                self.vertex_buffer,
                self.index_buffer,
                0..3,
            )
            .submit();
        self.swapchain.copy_and_present(self.color_image);
    }
}

fn main() {
    unsafe {
        let ctx = tephra_vulkan::Context::new();
        let triangle = Triangle::new(&ctx).unwrap();
        let mut pool = Pool::new(&ctx);
        loop {
            let mut command_list = CommandList::new();
            triangle.record_commands(&mut command_list);
            ctx.submit_commands(&mut pool, &command_list);
            pool.reset();
        }
    }
}
