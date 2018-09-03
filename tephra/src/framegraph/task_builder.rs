use super::{Access, Framegraph, Handle, Recording, Resource, ResourceAccess, ResourceType};
use image::{Image, ImageDesc};
pub struct TaskBuilder<'graph> {
    pub(crate) pass_handle: Handle,
    pub(crate) framegraph: &'graph mut Framegraph<Recording>,
}
impl<'graph> TaskBuilder<'graph> {
    pub fn create_image(
        &mut self,
        name: &'static str,
        desc: ImageDesc,
    ) -> Resource<Image> {
        // TODO: Freeze resources or this ins incorrect
        let id = self.framegraph.resources.len() + self.framegraph.state.image_data.len();
        self.framegraph.state.image_data.push((id, desc));
        let resource = Resource::new(id, 0);
        self.framegraph
            .insert_pass_handle(resource, self.pass_handle);
        resource
    }

    pub fn write<T>(
        &mut self,
        resource: Resource<T>,
    ) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Write,
        };
        let handle = self.framegraph.get_pass_handle(resource).expect("Handle");
        self.framegraph
            .graph
            .add_edge(handle, self.pass_handle, access);
        let write_resource = Resource::new(resource.id, resource.version + 1);
        self.framegraph
            .insert_pass_handle(write_resource, self.pass_handle);
        write_resource
    }

    pub fn read<T>(
        &mut self,
        resource: Resource<T>,
    ) -> Resource<T> {
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
