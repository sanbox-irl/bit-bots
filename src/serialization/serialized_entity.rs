use super::{
    physics_components::*, prefab_system, ComponentBounds, ComponentDatabase, ConversantNPC, DrawRectangle,
    Entity, Follow, GraphNode, GridObject, Marker, Name, NonInspectableEntities, Player, PrefabMarker,
    ResourcesDatabase, SceneSwitcher, SerializableComponent, SingletonDatabase, SoundSource, Sprite,
    TextSource, Transform, Velocity,
};
use serde_yaml::Value as YamlValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct SerializedComponent<T> {
    pub inner: T,
    pub active: bool,
}

pub type SerializedComponentWrapper<T> = Option<SerializedComponent<T>>;

pub trait SerializedComponentExtenstions<T: Default> {
    fn unwrap_or_else_create<F: FnOnce() -> SerializedComponent<T>>(
        &mut self,
        f: F,
    ) -> &mut SerializedComponent<T>;

    fn unwrap_or_create_default(&mut self) -> &mut SerializedComponent<T>;
}

impl<T: Default> SerializedComponentExtenstions<T> for SerializedComponentWrapper<T> {
    fn unwrap_or_else_create<F: FnOnce() -> SerializedComponent<T>>(
        &mut self,
        f: F,
    ) -> &mut SerializedComponent<T> {
        if let Some(inner) = self {
            inner
        } else {
            *self = Some(f());
            self.as_mut().unwrap()
        }
    }
    fn unwrap_or_create_default(&mut self) -> &mut SerializedComponent<T> {
        if let Some(inner) = self {
            inner
        } else {
            *self = Some(SerializedComponent {
                inner: T::default(),
                active: true,
            });
            self.as_mut().unwrap()
        }
    }
}

// This should mirror ComponentDatabse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct SerializedEntity {
    pub name: SerializedComponentWrapper<Name>,
    pub player: SerializedComponentWrapper<Player>,
    pub transform: SerializedComponentWrapper<Transform>,
    pub grid_object: SerializedComponentWrapper<GridObject>,
    pub scene_switcher: SerializedComponentWrapper<SceneSwitcher>,
    pub graph_node: SerializedComponentWrapper<GraphNode>,
    pub velocity: SerializedComponentWrapper<Velocity>,
    pub sprite: SerializedComponentWrapper<Sprite>,
    pub sound_source: SerializedComponentWrapper<SoundSource>,
    pub draw_rectangle: SerializedComponentWrapper<DrawRectangle>,
    pub bounding_box: SerializedComponentWrapper<BoundingBox>,
    pub text_source: SerializedComponentWrapper<TextSource>,
    // pub tilemap: SerializedComponentWrapper<TilemapSerialized>,
    pub follow: SerializedComponentWrapper<Follow>,
    pub conversant_npc: SerializedComponentWrapper<ConversantNPC>,
    pub prefab_marker: SerializedComponentWrapper<PrefabMarker>,

    pub id: Uuid,
    pub marker: Option<Marker>,
}

impl SerializedEntity {
    pub fn new(
        entity_id: &Entity,
        serialization_id: Uuid,
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
        resources: &ResourcesDatabase,
    ) -> Option<Self> {
        let mut prefab = None;

        let mut serialized_entity = SerializedEntity::with_prefab_components(
            entity_id,
            serialization_id,
            component_database,
            singleton_database,
            resources,
            Some(&mut prefab),
        )?;

        if let Some(prefab) = prefab {
            serialized_entity
                .foreach_component_dedup(|component, active| component.is_serialized(&prefab, *active));
        }

        Some(serialized_entity)
    }

    /// This creates a serialized entity, but it will not dedup the prefab components out. If the entity
    /// is a prefab inheritor, then this will not be the same as what is written to disk, but **will be**
    /// the same as what is currently *live* (as far as SE == live entities goes, that is)
    pub fn with_prefab_components(
        entity_id: &Entity,
        serialization_id: Uuid,
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
        resources: &ResourcesDatabase,
        give_prefab: Option<&mut Option<SerializedEntity>>,
    ) -> Option<SerializedEntity> {
        // If it's a prefab, add in all the PREFAB components
        let mut serialized_entity = Default::default();

        let loaded_prefab = prefab_system::get_serialized_parent_prefab_from_inheritor(
            component_database.prefab_markers.get(entity_id),
            resources,
            &mut serialized_entity,
        );

        if loaded_prefab {
            // If the caller want the original prefab too, they can have it.
            if let Some(give_prefab) = give_prefab {
                *give_prefab = Some(serialized_entity.clone());
            }
        }

        // Save over the ID at this stage (we probably had the prefab ID in there)
        serialized_entity.id = serialization_id;

        // Now add in all the normal components:
        component_database.foreach_component_list(
            NonInspectableEntities::NAME | NonInspectableEntities::GRAPH_NODE,
            |component_list| {
                component_list.load_component_into_serialized_entity(
                    entity_id,
                    &mut serialized_entity,
                    &component_database.serialization_markers,
                );
            },
        );
        serialized_entity.marker = singleton_database.save_singleton_markers(entity_id);

        Some(serialized_entity)
    }

    pub fn new_blank() -> Self {
        SerializedEntity {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn with_uuid(uuid: Uuid) -> Self {
        SerializedEntity {
            id: uuid,
            ..Default::default()
        }
    }

    pub fn foreach_component(
        &mut self,
        entity_bitmask: NonInspectableEntities,
        mut f: impl FnMut(&dyn ComponentBounds, &bool),
        f_util: Option<impl FnMut(&mut Uuid, &mut Option<Marker>)>,
    ) {
        let SerializedEntity {
            name,
            player,
            transform,
            grid_object,
            scene_switcher,
            graph_node,
            velocity,
            sprite,
            sound_source,
            draw_rectangle,
            bounding_box,
            text_source,
            // tilemap,
            follow,
            conversant_npc,
            prefab_marker,
            id,
            marker,
        } = self;

        macro_rules! do_action {
            ( $( $x:ident ),* ) => {
                $(
                    if let Some(serialized_component) = $x {
                        f(&serialized_component.inner, &serialized_component.active);
                    }
                )*
            };
        }

        if entity_bitmask.contains(NonInspectableEntities::NAME) {
            do_action!(name);
        }

        do_action!(
            player,
            transform,
            grid_object,
            scene_switcher,
            velocity,
            sprite,
            sound_source,
            draw_rectangle,
            bounding_box,
            text_source,
            // tilemap,
            follow,
            conversant_npc
        );

        if entity_bitmask.contains(NonInspectableEntities::GRAPH_NODE) {
            do_action!(graph_node);
        }

        if entity_bitmask.contains(NonInspectableEntities::PREFAB) {
            do_action!(prefab_marker);
        }

        if let Some(mut f_util) = f_util {
            f_util(id, marker);
        }
    }

    pub fn foreach_component_dedup(&mut self, mut f: impl FnMut(&dyn ComponentBounds, &bool) -> bool) {
        let SerializedEntity {
            name,
            player,
            transform,
            grid_object,
            scene_switcher,
            graph_node,
            velocity,
            sprite,
            sound_source,
            draw_rectangle,
            bounding_box,
            text_source,
            // tilemap,
            follow,
            conversant_npc,
            prefab_marker: _,
            id: _,
            marker: _,
        } = self;

        macro_rules! dedup_serialization {
            ( $( $x:ident ),* ) => {
                $(
                    if let Some(serialized_component) = $x {
                        if f(&serialized_component.inner, &serialized_component.active) {
                            *$x = None;
                        }
                    }
                )*
            };
        }

        dedup_serialization!(
            name,
            player,
            transform,
            grid_object,
            scene_switcher,
            graph_node,
            velocity,
            sprite,
            sound_source,
            draw_rectangle,
            bounding_box,
            text_source,
            // tilemap,
            follow,
            conversant_npc
        );
    }

    pub fn log_to_console(&self) {
        println!("---");
        println!("Serialized Entity: {:#?}", self);
        println!("---");
    }

    pub fn get_serialized_yaml_component<T: SerializableComponent + ComponentBounds>(
        serialized_entity: &SerializedEntity,
    ) -> YamlValue {
        if let YamlValue::Mapping(mut serialized_entity_value) =
            serde_yaml::to_value(serialized_entity.clone()).unwrap()
        {
            serialized_entity_value
                .remove(&T::SERIALIZATION_NAME)
                .unwrap_or_default()
        } else {
            YamlValue::default()
        }
    }

    pub fn get_serialized_component<T: SerializableComponent + ComponentBounds>(
        serialized_entity: &SerializedEntity,
    ) -> Option<SerializedComponent<T>> {
        let my_output = SerializedEntity::get_serialized_yaml_component::<T>(serialized_entity);
        if my_output.is_mapping() {
            serde_yaml::from_value(my_output).unwrap_or_default()
        } else {
            None
        }
    }

    pub fn create_yaml_component<T: SerializableComponent + ComponentBounds>(
        cmp: &super::Component<T>,
    ) -> YamlValue {
        serde_yaml::to_value(SerializedComponent {
            inner: cmp.clone_inner(),
            active: cmp.is_active,
        })
        .unwrap_or_default()
    }
}
