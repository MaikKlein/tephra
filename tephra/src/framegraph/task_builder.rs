use super::{Access, Framegraph, Handle, Recording, Resource, ResourceAccess, ResourceType};
use image::{Image, ImageDesc};
pub struct TaskBuilder<'graph> {
    pub(crate) pass_handle: Handle,
    pub(crate) framegraph: &'graph mut Framegraph<Recording>,
}
impl<'graph> TaskBuilder<'graph> {
    pub fn create_image(&mut self, name: &'static str, desc: ImageDesc) -> Resource<Image> {
        self.framegraph.state.image_data.push(desc);
        let id = self.framegraph.state.image_data.len() - 1;
        Resource::new(name, id, self.pass_handle)
    }

    pub fn write<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Write,
            ty: ResourceType::Image,
        };
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, access);
        Resource::new(resource.name, resource.id, self.pass_handle)
    }

    pub fn read<T>(&mut self, resource: Resource<T>) -> Resource<T> {
        let access = Access {
            resource: resource.id,
            resource_access: ResourceAccess::Read,
            ty: ResourceType::Image,
        };
        self.framegraph
            .graph
            .add_edge(resource.handle, self.pass_handle, access);
        resource
    }
}
