use std::mem;
use super::Vulkan;
use ash::vk;
use context::Context;
use pipeline::{PipelineApi, PipelineBuilder};
use std::ptr;
use std::ffi::CString;
pub struct PipelineData {
    pipeline: vk::Pipeline,
}
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::uninitialized();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}
impl PipelineApi<Vulkan> for PipelineData {
    fn from_pipeline_builder(
        context: &Context<Vulkan>,
        pipline_builder: &PipelineBuilder<Vulkan>,
    ) -> Self {
        // let layout_create_info = vk::PipelineLayoutCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     set_layout_count: 0,
        //     p_set_layouts: ptr::null(),
        //     push_constant_range_count: 0,
        //     p_push_constant_ranges: ptr::null(),
        // };

        // let pipeline_layout = context
        //     .device
        //     .create_pipeline_layout(&layout_create_info, None)
        //     .unwrap();

        // let shader_entry_name = CString::new("main").unwrap();
        // let shader_stage_create_infos = [
        //     vk::PipelineShaderStageCreateInfo {
        //         s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        //         p_next: ptr::null(),
        //         flags: Default::default(),
        //         module: vertex_shader_module.shader_data.shader_module,
        //         p_name: shader_entry_name.as_ptr(),
        //         p_specialization_info: ptr::null(),
        //         stage: vk::ShaderStageFlags::VERTEX,
        //     },
        //     vk::PipelineShaderStageCreateInfo {
        //         s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        //         p_next: ptr::null(),
        //         flags: Default::default(),
        //         module: fragment_shader_module.shader_data.shader_module,
        //         p_name: shader_entry_name.as_ptr(),
        //         p_specialization_info: ptr::null(),
        //         stage: vk::ShaderStageFlags::FRAGMENT,
        //     },
        // ];
        // let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
        //     binding: 0,
        //     stride: mem::size_of::<Vertex>() as u32,
        //     input_rate: vk::VertexInputRate::VERTEX,
        // }];
        // let vertex_input_attribute_descriptions = [
        //     vk::VertexInputAttributeDescription {
        //         location: 0,
        //         binding: 0,
        //         format: vk::Format::R32G32B32A32_SFLOAT,
        //         offset: offset_of!(Vertex, pos) as u32,
        //     },
        //     vk::VertexInputAttributeDescription {
        //         location: 1,
        //         binding: 0,
        //         format: vk::Format::R32G32B32A32_SFLOAT,
        //         offset: offset_of!(Vertex, color) as u32,
        //     },
        // ];
        // let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
        //     p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
        //     vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
        //     p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
        // };
        // let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        //     flags: Default::default(),
        //     p_next: ptr::null(),
        //     primitive_restart_enable: 0,
        //     topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        // };
        // let viewports = [vk::Viewport {
        //     x: 0.0,
        //     y: 0.0,
        //     width: context.surface_resolution.width as f32,
        //     height: context.surface_resolution.height as f32,
        //     min_depth: 0.0,
        //     max_depth: 1.0,
        // }];
        // let scissors = [vk::Rect2D {
        //     offset: vk::Offset2D { x: 0, y: 0 },
        //     extent: context.surface_resolution.clone(),
        // }];
        // let viewport_state_info = vk::PipelineViewportStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     scissor_count: scissors.len() as u32,
        //     p_scissors: scissors.as_ptr(),
        //     viewport_count: viewports.len() as u32,
        //     p_viewports: viewports.as_ptr(),
        // };
        // let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     cull_mode: vk::CullModeFlags::NONE,
        //     depth_bias_clamp: 0.0,
        //     depth_bias_constant_factor: 0.0,
        //     depth_bias_enable: 0,
        //     depth_bias_slope_factor: 0.0,
        //     depth_clamp_enable: 0,
        //     front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        //     line_width: 1.0,
        //     polygon_mode: vk::PolygonMode::FILL,
        //     rasterizer_discard_enable: 0,
        // };
        // let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        //     flags: Default::default(),
        //     p_next: ptr::null(),
        //     rasterization_samples: vk::SampleCountFlags::TYPE_1,
        //     sample_shading_enable: 0,
        //     min_sample_shading: 0.0,
        //     p_sample_mask: ptr::null(),
        //     alpha_to_one_enable: 0,
        //     alpha_to_coverage_enable: 0,
        // };
        // let noop_stencil_state = vk::StencilOpState {
        //     fail_op: vk::StencilOp::KEEP,
        //     pass_op: vk::StencilOp::KEEP,
        //     depth_fail_op: vk::StencilOp::KEEP,
        //     compare_op: vk::CompareOp::ALWAYS,
        //     compare_mask: 0,
        //     write_mask: 0,
        //     reference: 0,
        // };
        // let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     depth_test_enable: 1,
        //     depth_write_enable: 1,
        //     depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        //     depth_bounds_test_enable: 0,
        //     stencil_test_enable: 0,
        //     front: noop_stencil_state.clone(),
        //     back: noop_stencil_state.clone(),
        //     max_depth_bounds: 1.0,
        //     min_depth_bounds: 0.0,
        // };
        // let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        //     blend_enable: 0,
        //     src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        //     dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        //     color_blend_op: vk::BlendOp::ADD,
        //     src_alpha_blend_factor: vk::BlendFactor::ZERO,
        //     dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        //     alpha_blend_op: vk::BlendOp::ADD,
        //     color_write_mask: vk::ColorComponentFlags::all(),
        // }];
        // let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     logic_op_enable: 0,
        //     logic_op: vk::LogicOp::CLEAR,
        //     attachment_count: color_blend_attachment_states.len() as u32,
        //     p_attachments: color_blend_attachment_states.as_ptr(),
        //     blend_constants: [0.0, 0.0, 0.0, 0.0],
        // };
        // let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        // let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        //     s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     dynamic_state_count: dynamic_state.len() as u32,
        //     p_dynamic_states: dynamic_state.as_ptr(),
        // };
        // let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
        //     s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: vk::PipelineCreateFlags::empty(),
        //     stage_count: shader_stage_create_infos.len() as u32,
        //     p_stages: shader_stage_create_infos.as_ptr(),
        //     p_vertex_input_state: &vertex_input_state_info,
        //     p_input_assembly_state: &vertex_input_assembly_state_info,
        //     p_tessellation_state: ptr::null(),
        //     p_viewport_state: &viewport_state_info,
        //     p_rasterization_state: &rasterization_info,
        //     p_multisample_state: &multisample_state_info,
        //     p_depth_stencil_state: &depth_state_info,
        //     p_color_blend_state: &color_blend_state,
        //     p_dynamic_state: &dynamic_state_info,
        //     layout: pipeline_layout,
        //     render_pass: renderpass,
        //     subpass: 0,
        //     base_pipeline_handle: vk::Pipeline::null(),
        //     base_pipeline_index: 0,
        // };
        // let graphics_pipelines = context
        //     .device
        //     .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None)
        //     .expect("Unable to create graphics pipeline");

        // let graphic_pipeline = graphics_pipelines[0];
        unimplemented!()
    }
}
