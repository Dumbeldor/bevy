use crate::{
    asset::{AssetStorage, Handle, Mesh},
    legion::prelude::*,
    render::render_graph::{
        resource_name, NewDrawTarget, PipelineDescriptor, RenderPass, Renderable, ResourceInfo,
        ShaderPipelineAssignments,
    },
};

#[derive(Default)]
pub struct AssignedMeshesDrawTarget;

impl NewDrawTarget for AssignedMeshesDrawTarget {
    fn draw(
        &self,
        world: &World,
        render_pass: &mut dyn RenderPass,
        pipeline_handle: Handle<PipelineDescriptor>,
    ) {
        let shader_pipeline_assignments =
            world.resources.get::<ShaderPipelineAssignments>().unwrap();
        let mut current_mesh_handle = None;
        let mut current_mesh_index_len = 0;

        let assigned_entities = shader_pipeline_assignments
            .assignments
            .get(&pipeline_handle);

        if let Some(assigned_entities) = assigned_entities {
            for entity in assigned_entities.iter() {
                // TODO: hopefully legion has better random access apis that are more like queries?
                let renderable = world.get_component::<Renderable>(*entity).unwrap();
                let mesh = *world.get_component::<Handle<Mesh>>(*entity).unwrap();
                if !renderable.is_visible {
                    continue;
                }

                let renderer = render_pass.get_renderer();
                let render_resources = renderer.get_render_resources();
                if current_mesh_handle != Some(mesh) {
                    if let Some(vertex_buffer_resource) =
                        render_resources.get_mesh_vertices_resource(mesh)
                    {
                        let index_buffer_resource =
                            render_resources.get_mesh_indices_resource(mesh).unwrap();
                        match renderer.get_resource_info(index_buffer_resource).unwrap() {
                            ResourceInfo::Buffer { size, .. } => {
                                current_mesh_index_len = (size / 2) as u32
                            }
                            _ => panic!("expected a buffer type"),
                        }
                        render_pass.set_index_buffer(index_buffer_resource, 0);
                        render_pass.set_vertex_buffer(0, vertex_buffer_resource, 0);
                    }
                    // TODO: Verify buffer format matches render pass
                    current_mesh_handle = Some(mesh);
                }

                // TODO: validate bind group properties against shader uniform properties at least once
                render_pass.set_bind_groups(Some(&entity));
                render_pass.draw_indexed(0..current_mesh_index_len, 0, 0..1);
            }
        }
    }

    fn setup(
        &mut self,
        world: &World,
        renderer: &mut dyn crate::render::render_graph::Renderer,
        pipeline_handle: Handle<PipelineDescriptor>,
    ) {
        let shader_pipeline_assignments =
            world.resources.get::<ShaderPipelineAssignments>().unwrap();
        let assigned_entities = shader_pipeline_assignments
            .assignments
            .get(&pipeline_handle);
        let pipeline_storage = world
            .resources
            .get::<AssetStorage<PipelineDescriptor>>()
            .unwrap();
        let pipeline_descriptor = pipeline_storage.get(&pipeline_handle).unwrap();
        if let Some(assigned_entities) = assigned_entities {
            for entity in assigned_entities.iter() {
                // TODO: hopefully legion has better random access apis that are more like queries?
                let renderable = world.get_component::<Renderable>(*entity).unwrap();
                if !renderable.is_visible {
                    continue;
                }

                renderer.setup_entity_bind_groups(*entity, pipeline_descriptor);
            }
        }
    }

    fn get_name(&self) -> String {
        resource_name::draw_target::ASSIGNED_MESHES.to_string()
    }
}