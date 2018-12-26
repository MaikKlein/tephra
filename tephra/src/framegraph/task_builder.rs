use crate::{
    buffer::Buffer,
    framegraph::{Access, Framegraph, Handle, Recording, Resource, ResourceAccess},
    renderpass::{Framebuffer, Renderpass},
};
pub mod deferred {
    use super::TaskBuilder;
    use crate::{
        framegraph::Resource,
        image::{Image, ImageDescBuilder},
    };

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
}

use crate::image::{Image, ImageDesc};
pub struct TaskBuilder<'frame> {
    pub pass_handle: Handle,
    pub framegraph: &'frame mut Framegraph<Recording>,
}
impl<'frame> TaskBuilder<'frame> {
    pub fn add_buffer<T>(&mut self, buffer: Buffer<T>) -> Resource<Buffer<T>> {
        let resource = self.framegraph.registry.add_buffer(buffer);
        self.framegraph
            .insert_pass_handle(resource, self.pass_handle);
        resource
    }
    pub fn create_framebuffer(
        &mut self,
        renderpass: Renderpass,
        images: Vec<Resource<Image>>,
    ) -> Resource<Framebuffer> {
        let id = self.framegraph.registry.reserve_index();
        self.framegraph
            .state
            .framebuffer_data
            .push((id, (renderpass, images)));
        let resource = Resource::new(id, 0);
        self.framegraph
            .insert_pass_handle(resource, self.pass_handle);
        resource
    }
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
