use super::{
    components::{ComponentDatabase, Entity},
    components_singleton::SingletonDatabase,
    entities::EntityAllocator,
    hardware_interfaces::HardwareInterface,
    resources::{PrefabMap, ResourcesDatabase},
    scene_graph::SceneGraph,
    systems::*,
    ActionMap, GameWorldDrawCommands,
};
use anyhow::Error;

pub struct Ecs {
    pub component_database: ComponentDatabase,
    pub scene_graph: SceneGraph,
    pub singleton_database: SingletonDatabase,
    pub entities: Vec<Entity>,
    pub entity_allocator: EntityAllocator,
}

impl Ecs {
    pub fn new(prefabs: &PrefabMap) -> Result<Self, Error> {
        // Es and Cs
        let mut entity_allocator = EntityAllocator::new();
        let mut entities = Vec::new();

        // Deserialize Entities and Singletons and load in the SceneGraph
        let mut marker_map = std::collections::HashMap::new();
        let mut scene_graph = SceneGraph::new();
        let component_database = ComponentDatabase::new(
            &mut entity_allocator,
            &mut entities,
            &mut marker_map,
            prefabs,
            &mut scene_graph,
        )?;

        let singleton_database = SingletonDatabase::new(marker_map)?;

        Ok(Ecs {
            entities,
            scene_graph,
            entity_allocator,
            component_database,
            singleton_database,
        })
    }

    /// The difference between GameStart and New is that everyting in initialized by now.
    pub fn game_start(
        &mut self,
        resources: &ResourcesDatabase,
        hardware_interfaces: &HardwareInterface,
    ) -> Result<(), Error> {
        self.singleton_database
            .initialize_with_runtime_resources(resources, hardware_interfaces);

        // tilemap_system::initialize_tilemaps(&mut self.component_database.tilemaps, &resources.tilesets);

        player_system::initialize_players(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
        );

        Ok(())
    }

    pub fn update(&mut self, actions: &ActionMap) -> Result<(), Error> {
        player_system::player_update(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
            &mut self.component_database.velocities,
            &mut self.scene_graph,
            actions,
        );

        Ok(())
    }

    pub fn update_resources(&mut self, resources: &ResourcesDatabase, delta_time: f32) {
        sprite_system::update_sprites(&mut self.component_database.sprites, resources, delta_time);
        cross_cutting_system::cross_cutting_system(self, resources);
    }

    pub fn render<'a, 'b>(
        &'a mut self,
        draw_commands: &'b mut DrawCommand<'a>,
        resources: &'a ResourcesDatabase,
    ) {
        draw_commands.game_world = Some(GameWorldDrawCommands {
            text_sources: &self.component_database.text_sources,
            sprites: &self.component_database.sprites,
            rects: &self.component_database.draw_rectangles,
            // tilemaps: &self.component_database.tilemaps,
            transforms: &self.component_database.transforms,
            camera_entity: self
                .singleton_database
                .associated_entities
                .get(&self.singleton_database.camera.marker()),
            camera: self.singleton_database.camera.inner(),
            rendering_utility: &mut self.singleton_database.rendering_utility,
            resources,
        })
    }
}

impl Ecs {
    /// This is the standard method to create a new Entity in the Ecs. Try to
    /// always use this one. The returned Entity is the ID, or index, of the new
    /// entity.
    pub fn create_entity(&mut self) -> Entity {
        Ecs::create_entity_raw(
            &mut self.component_database,
            &mut self.entity_allocator,
            &mut self.entities,
        )
    }

    /// For use during creation and startup, before we have an Ecs
    /// to do anything with
    pub fn remove_entity_raw(
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
        component_database: &mut ComponentDatabase,
        scene_graph: &SceneGraph,
        entity_to_delete: &Entity,
    ) -> bool {
        let is_dealloc = entity_allocator.deallocate(entity_to_delete);
        if is_dealloc {
            component_database.deregister_entity(&entity_to_delete, scene_graph);
            entities
                .iter()
                .position(|i| i == entity_to_delete)
                .map(|i| entities.remove(i));
        }
        is_dealloc
    }

    /// Use this only in weird situations. Otherwise, prefer to pass
    /// the entire Ecs around now that we have a leaner top level
    /// struct.
    pub fn create_entity_raw(
        component_database: &mut ComponentDatabase,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
    ) -> Entity {
        let entity = entity_allocator.allocate();
        component_database.register_entity(entity);
        entities.push(entity);
        entity
    }

    pub fn remove_entity(&mut self, entity_to_delete: &Entity) -> bool {
        // If it's in the SceneGraph, we're going to delete its children too.
        // children are, and this is a fact, the worst. Jk, I wnat to be a father one day.
        if let Some(transform) = self.component_database.transforms.get(entity_to_delete) {
            if let Some(node_id) = transform.inner().scene_graph_node_id() {
                let scene_graph_raw_handle: *mut SceneGraph = &mut self.scene_graph;
                for descendant in node_id
                    .descendants(&self.scene_graph)
                    .collect::<Vec<_>>()
                    .iter()
                    .rev()
                {
                    if descendant == &node_id {
                        continue;
                    }

                    let id = self.scene_graph.get(*descendant).unwrap().inner();
                    Ecs::remove_entity_raw(
                        &mut self.entity_allocator,
                        &mut self.entities,
                        &mut self.component_database,
                        &self.scene_graph,
                        id,
                    );
                    descendant.remove(unsafe { &mut *scene_graph_raw_handle });
                }
                node_id.remove(&mut self.scene_graph);
            }
        }

        Ecs::remove_entity_raw(
            &mut self.entity_allocator,
            &mut self.entities,
            &mut self.component_database,
            &mut self.scene_graph,
            entity_to_delete,
        )
    }

    pub fn clone_entity(&mut self, original: &Entity) -> Entity {
        let new_entity = self.create_entity();
        self.component_database
            .clone_components(original, &new_entity, &mut self.scene_graph);

        new_entity
    }
}
