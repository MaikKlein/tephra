use super::{Context, Vulkan};
use ash::vk;
use commandbuffer::{self, Compute, CreateExecute, ExecuteApi, Graphics, GraphicsCmd};
use std::ptr;

pub struct Commandbuffer {
    ctx: Context,
}

// impl CommandbufferApi for Commandbuffer<Graphics> {
// }
pub struct Execute {
    ctx: Context,
}
impl ExecuteApi for Execute {
    fn execute_commands(&self, cmds: &[GraphicsCmd]) {
        let _clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        //let mut pipelines = Vec::new();
        super::CommandBuffer::record(&self.ctx, "Execute", |_cb| {
            let _device = &self.ctx.device;
            let mut _outer_render: Option<&super::render::Render> = None;
            for cmd in cmds {
                match cmd {
                    GraphicsCmd::BindPipeline {
                        state: _,
                        stride: _,
                        vertex_input_data: _,
                    } => {
                        unsafe {
                            // let pipeline = super::render::create_pipeline(
                            //     &self.ctx,
                            //     state,
                            //     *stride,
                            //     &vertex_input_data,
                            //     outer_render.unwrap().renderpass,
                            //     );
                            // pipelines.push(pipeline);
                        }
                    }
                    _ => (),
                }
            }
        });
    }
}
impl CreateExecute for Context {
    fn create_execute(&self) -> commandbuffer::Execute {
        let execute = Execute { ctx: self.clone() };
        commandbuffer::Execute {
            inner: Box::new(execute),
        }
    }
}
