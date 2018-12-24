use crate::{
    context::Context,
    framegraph::{Access, Framegraph, Handle, Recording, Resource, ResourceAccess, ResourceType},
    renderpass::{Attachment, Attachments, RenderTarget, RenderTargetState},
};
pub mod deferred {
    use super::TaskBuilder;
    use crate::{
        descriptor::{Binding, DescriptorInfo, DescriptorResource, DescriptorType},
        framegraph::{Registry, Resource},
        image::{Image, ImageDescBuilder},
        pipeline::{self, ShaderStage},
        renderpass::{self, RenderTarget, VertexInput, VertexInputData},
    };
    use derive_builder::Builder;
    use smallvec::SmallVec;

    impl Image {
        pub fn deferred() -> ImageDescBuilder {
            Default::default()
        }
    }
    impl ImageDescBuilder {
        pub fn build_deferred<'task>(self, builder: &mut TaskBuilder<'task>) -> Resource<Image> {
            let id = builder.framegraph.registry.reserve_index();
            let image_desc = self.build().unwrap();
            builder.framegraph.state.image_data.push((id, image_desc));
            let resource = Resource::new(id, 0);
            builder
                .framegraph
                .insert_pass_handle(resource, builder.pass_handle);
            resource
        }
    }
    pub type Stride = u32;
    #[derive(Builder)]
    #[builder(pattern = "owned")]
    pub struct PipelineState {
        pub vertex_shader: ShaderStage,
        pub fragment_shader: ShaderStage,
        pub render_target: Resource<RenderTarget>,
        #[builder(setter(skip = "false"))]
        pub layout: Vec<Binding<DescriptorType>>,
        #[builder(setter(skip = "false"))]
        // TODO: Default to SoA not AoS
        pub vertex_input: (Stride, Vec<VertexInputData>),
    }
    impl pipeline::PipelineState {
        pub fn deferred() -> PipelineStateBuilder {
            Default::default()
        }
    }
    impl PipelineState {
        pub fn into_non_deferred(self, registry: &Registry) -> pipeline::PipelineState {
            pipeline::PipelineState {
                vertex_shader: self.vertex_shader,
                fragment_shader: self.fragment_shader,
                render_target: registry.get_render_target(self.render_target),
                layout: self.layout,
                vertex_input: self.vertex_input,
            }
        }
    }

    impl PipelineStateBuilder {
        pub fn build_deferred<'task>(
            self,
            builder: &mut TaskBuilder<'task>,
        ) -> Resource<PipelineState> {
            let id = builder.framegraph.registry.reserve_index();
            let pipeline_state = self.build().unwrap();
            builder
                .framegraph
                .state
                .pipeline_states
                .push((id, pipeline_state));
            let resource = Resource::new(id, 0);
            builder
                .framegraph
                .insert_pass_handle(resource, builder.pass_handle);
            resource
        }

        pub fn layout<D: DescriptorInfo>(mut self) -> Self {
            self.layout = Some(D::layout());
            self
        }
        pub fn vertex<V: VertexInput>(mut self) -> Self {
            self.vertex_input = Some((std::mem::size_of::<V>() as Stride, V::vertex_input_data()));
            self
        }
    }

    #[derive(Builder)]
    pub struct Attachment {
        pub image: Resource<Image>,
        pub index: u32,
    }

    impl Attachment {
        pub fn builder() -> AttachmentBuilder {
            AttachmentBuilder::default()
        }
    }

    pub type Attachments = SmallVec<[Attachment; 10]>;
    #[derive(Default)]
    pub struct RenderTargetState {
        pub color_attachments: Attachments,
        pub depth_attachment: Option<Attachment>,
    }
    impl RenderTarget {
        pub fn deferred() -> RenderTargetBuilder {
            Default::default()
        }
    }
    impl RenderTargetState {
        pub fn into_non_deferred(self, registry: &Registry) -> renderpass::RenderTargetState {
            let color_attachments: renderpass::Attachments = self
                .color_attachments
                .into_iter()
                .map(|attachment| renderpass::Attachment {
                    image: registry.get_image(attachment.image),
                    index: attachment.index,
                })
                .collect();
            let depth_attachment = self
                .depth_attachment
                .map(|attachment| renderpass::Attachment {
                    image: registry.get_image(attachment.image),
                    index: attachment.index,
                });
            renderpass::RenderTargetState {
                color_attachments,
                depth_attachment,
            }
        }
    }
    #[derive(Default)]
    pub struct RenderTargetBuilder {
        state: RenderTargetState,
    }

    impl RenderTargetBuilder {
        pub fn color_attachment(mut self, attachment: Attachment) -> Self {
            self.state.color_attachments.push(attachment);
            self
        }
        pub fn set_depth_attachment(mut self, attachment: Attachment) -> Self {
            self.state.depth_attachment = Some(attachment);
            self
        }

        pub fn build_deferred<'task>(
            self,
            builder: &mut TaskBuilder<'task>,
        ) -> Resource<RenderTarget> {
            let id = builder.framegraph.registry.reserve_index();
            builder
                .framegraph
                .state
                .render_targets
                .push((id, self.state));
            let resource = Resource::new(id, 0);
            builder
                .framegraph
                .insert_pass_handle(resource, builder.pass_handle);
            resource
        }
    }
}

use crate::image::{Image, ImageDesc};
pub struct TaskBuilder<'frame> {
    pub(crate) pass_handle: Handle,
    pub(crate) framegraph: &'frame mut Framegraph<Recording>,
}
impl<'frame> TaskBuilder<'frame> {
    pub fn create_image(&mut self, _name: &'static str, desc: ImageDesc) -> Resource<Image> {
        let id = self.framegraph.registry.reserve_index();
        self.framegraph.state.image_data.push((id, desc));
        let resource = Resource::new(id, 0);
        self.framegraph
            .insert_pass_handle(resource, self.pass_handle);
        resource
    }

    pub fn write<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Write,
        };
        if let Some(handle) = self.framegraph.get_pass_handle(resource) {
            self.framegraph
                .graph
                .add_edge(handle, self.pass_handle, access);
        }
        let write_resource = Resource::new(resource.id, resource.version + 1);
        self.framegraph
            .insert_pass_handle(write_resource, self.pass_handle);
        write_resource
    }

    pub fn read<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Read,
        };
        let handle = self.framegraph.get_pass_handle(resource).expect("Handle");
        self.framegraph
            .graph
            .add_edge(handle, self.pass_handle, access);
        resource
    }
}
