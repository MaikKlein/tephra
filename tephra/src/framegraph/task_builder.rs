use crate::{
    context::Context,
    framegraph::{Access, Framegraph, Handle, Recording, Resource, ResourceAccess, ResourceType},
    renderpass::{Attachment, Attachments, RenderTarget},
};
pub struct RenderTargetBuilder<'a> {
    recording: &'a mut Recording,
    state: RenderTargetState,
}
pub struct RenderTargetState {
    pub color_attachments: Attachments,
    pub depth_attachment: Option<Attachment>,
}

impl RenderTargetBuilder<'_> {
    pub fn color_attachment(self) -> Self {
        self
    }
    pub fn depth_attachment(self) -> Self {
        self
    }

    pub fn build(self) -> Resource<RenderTarget> {
        unimplemented!()
    }
}
use crate::image::{Image, ImageDesc};
pub struct TaskBuilder<'borrow> {
    pub(crate) pass_handle: Handle,
    pub(crate) framegraph: &'borrow mut Framegraph<Recording>,
}
impl<'borrow> TaskBuilder<'borrow> {
    pub fn create_render_target(&mut self) -> RenderTargetBuilder<'borrow> {
        unimplemented!()
    }
    pub fn create_image(&mut self, _name: &'static str, desc: ImageDesc) -> Resource<Image> {
        // TODO: Freeze resources or this is incorrect
        let id = self.framegraph.resources.len() + self.framegraph.state.image_data.len();
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
