use super::Context;
use crate::{
    descriptor::{Binding, DescriptorType},
    pipeline::{
        ComputePipeline, ComputePipelineState, GraphicsPipeline, GraphicsPipelineState, PipelineApi,
    },
    renderpass::{VertexInputData, VertexType},
};
use ash::{version::DeviceV1_0, vk};
use std::{ffi::CString, ptr};

unsafe fn create_layout(
    ctx: &Context,
    data: &[Binding<DescriptorType>],
) -> Vec<vk::DescriptorSetLayout> {
    let layout_bindings: Vec<_> = data
        .iter()
        .map(|desc| {
            let ty = match desc.data {
                DescriptorType::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                DescriptorType::Storage => vk::DescriptorType::STORAGE_BUFFER,
            };
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: ty,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::ALL,
                p_immutable_samplers: std::ptr::null(),
            }
        })
        .collect();
    let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: layout_bindings.len() as u32,
        p_bindings: layout_bindings.as_ptr(),
        ..Default::default()
    };

    // TODO: Handle multiple layouts
    vec![ctx
        .device
        .create_descriptor_set_layout(&descriptor_info, None)
        .unwrap()]
}
pub unsafe fn create_pipeline_layout(
    ctx: &Context,
    layouts: &[vk::DescriptorSetLayout],
) -> vk::PipelineLayout {
    let layout_create_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: layouts.len() as _,
        p_set_layouts: layouts.as_ptr(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
        ..Default::default()
    };

    ctx.device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
}
impl PipelineApi for Context {
    unsafe fn create_compute_pipeline(&self, state: &ComputePipelineState) -> ComputePipeline {
        let vk_shader = self.shader_modules.get(state.compute_shader.shader_module);
        let shader_entry_name = CString::new(state.compute_shader.entry_name.as_str()).unwrap();
        let descriptor_layouts = create_layout(self, &state.layout);
        let pipeline_layout = create_pipeline_layout(self, &descriptor_layouts);
        let create_info = vk::ComputePipelineCreateInfo {
            layout: pipeline_layout,
            stage: vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::COMPUTE,
                module: vk_shader.shader_module,
                p_name: shader_entry_name.as_ptr(),
                ..Default::default()
            },
            ..Default::default()
        };
        let pipelines = self
            .device
            .create_compute_pipelines(self.pipeline_cache, &[create_info], None)
            .expect("pipeline");
        let data = ComputePipelineData {
            pipeline: pipelines[0],
            descriptor_layouts,
            layout: pipeline_layout,
        };
        self.compute_pipelines.insert(data)
    }
    unsafe fn create_graphics_pipeline(&self, state: &GraphicsPipelineState) -> GraphicsPipeline {
        let vertex_shader = &state.vertex_shader;
        let vk_vertex = self.shader_modules.get(state.vertex_shader.shader_module);
        let vk_fragment = self.shader_modules.get(state.fragment_shader.shader_module);

        let vertex_name = CString::new(vertex_shader.entry_name.as_str()).unwrap();
        let fragment_name = CString::new(state.fragment_shader.entry_name.as_str()).unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                module: vk_vertex.shader_module,
                p_name: vertex_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::VERTEX,
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                module: vk_fragment.shader_module,
                p_name: fragment_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: state.vertex_input.0,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = vertex_input(&state.vertex_input.1);
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags: Default::default(),
            p_next: ptr::null(),
            primitive_restart_enable: 0,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        };
        // TODO: Custom resolution
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.surface_resolution.width as f32,
            height: self.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.surface_resolution.clone(),
        }];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
        };
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            cull_mode: vk::CullModeFlags::NONE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: 0,
            depth_bias_slope_factor: 0.0,
            depth_clamp_enable: 0,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: 0,
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags: Default::default(),
            p_next: ptr::null(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: 0,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: 0,
            alpha_to_coverage_enable: 0,
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: noop_stencil_state.clone(),
            back: noop_stencil_state.clone(),
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::all(),
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::CLEAR,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_state.len() as u32,
            p_dynamic_states: dynamic_state.as_ptr(),
        };
        let descriptor_layouts = create_layout(self, &state.layout);
        let pipeline_layout = create_pipeline_layout(self, &descriptor_layouts);
        let render_target_data = self.renderpasses.get(state.render_target);
        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_create_infos.len() as u32,
            p_stages: shader_stage_create_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_state_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_info,
            p_rasterization_state: &rasterization_info,
            p_multisample_state: &multisample_state_info,
            p_depth_stencil_state: &depth_state_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: &dynamic_state_info,
            layout: pipeline_layout,
            render_pass: render_target_data.render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
        };
        let graphics_pipelines = self
            .device
            .create_graphics_pipelines(self.pipeline_cache, &[graphic_pipeline_info], None)
            .expect("Unable to create graphics pipeline");
        let data = GraphicsPipelineData {
            pipeline: graphics_pipelines[0],
            layout: pipeline_layout,
            descriptor_layouts,
        };
        self.graphic_pipelines.insert(data)
    }
}
pub fn vertex_format(ty: VertexType) -> vk::Format {
    match ty {
        VertexType::F32(size) => match size {
            1 => vk::Format::R32_SFLOAT,
            2 => vk::Format::R32G32_SFLOAT,
            3 => vk::Format::R32G32B32_SFLOAT,
            4 => vk::Format::R32G32B32A32_SFLOAT,
            _ => unreachable!(),
        },
    }
}
pub fn vertex_input(vertex_input: &[VertexInputData]) -> Vec<vk::VertexInputAttributeDescription> {
    vertex_input
        .iter()
        .map(|input| vk::VertexInputAttributeDescription {
            location: input.location,
            binding: input.binding,
            offset: input.offset,
            format: vertex_format(input.vertex_type),
        })
        .collect()
}
pub struct ComputePipelineData {
    pub pipeline: vk::Pipeline,
    // Maybe those should be destroyed after the pipeline has
    // been created?
    pub layout: vk::PipelineLayout,
    pub descriptor_layouts: Vec<vk::DescriptorSetLayout>,
}
pub struct GraphicsPipelineData {
    pub pipeline: vk::Pipeline,
    // Maybe those should be destroyed after the pipeline has
    // been created?
    pub layout: vk::PipelineLayout,
    pub descriptor_layouts: Vec<vk::DescriptorSetLayout>,
}
