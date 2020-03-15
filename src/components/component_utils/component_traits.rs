use super::{
    imgui_component_utils::*, scene_graph::SceneGraph, ComponentList, Entity, Name, PrefabMap, SceneMode,
    SerializedEntity, TrackedEntitiesMap,
};
use imgui::Ui;

pub trait ComponentBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
    fn is_serialized(&self, serialized_entity: &SerializedEntity, active: bool) -> bool;
    fn commit_to_scene(
        &self,
        serialized_entity: &mut SerializedEntity,
        active: bool,
        serialization_map: &TrackedEntitiesMap,
    );
    fn uncommit_to_scene(&self, serialized_entity: &mut SerializedEntity);

    #[inline]
    fn on_set(&mut self, _: &Entity, _: &mut SceneGraph) {}

    #[inline]
    fn on_unset(&mut self, _: Entity, _: &mut dyn ComponentListBounds, _: &SceneGraph) {}
}

pub trait ComponentPostDeserialization {
    #[inline]
    fn post_deserialization(&mut self, _: Entity, _: &TrackedEntitiesMap, _: &SceneGraph) {}
}

pub trait SerializableComponent:
    std::fmt::Debug
    + typename::TypeName
    + Clone
    + Default
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
    + 'static
{
    const SERIALIZATION_NAME: once_cell::sync::Lazy<serde_yaml::Value>;
}

pub trait ComponentListBounds {
    fn expand_list(&mut self);
    fn unset_component(&mut self, index: &Entity, scene_graph: &SceneGraph) -> bool;
    fn get_for_post_deserialization(
        &mut self,
        index: &Entity,
    ) -> Option<(&mut dyn ComponentPostDeserialization, bool)>;
    fn dump_to_log(&self, index: &Entity);
    fn clone_entity(&mut self, index: &Entity, new_entity: &Entity, scene_graph: &mut SceneGraph);

    // IMGUI
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>, scene_graph: &mut SceneGraph);
    fn component_inspector(
        &mut self,
        index: &Entity,
        scene_mode: SceneMode,
        prefab_sync_status: SyncStatus,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &imgui::Ui<'_>,
        is_open: bool,
    ) -> (Option<ComponentSerializationCommandType>, bool);

    // fn serialization_option(
    //     &self,
    //     ui: &Ui<'_>,
    //     index: &Entity,
    //     serialized_marker: &EntitySerializationMap,
    // ) -> Option<ComponentSerializationCommandType>;

    fn load_component_into_serialized_entity(
        &self,
        index: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &TrackedEntitiesMap,
    );

    fn post_deserialization(&mut self, entity_names: &TrackedEntitiesMap, scene_graph: &SceneGraph);

    // This gets the sync status of a given Entity, provided, optionally, two serialized entities.
    // In the most general sense, the returned SyncStatus
    fn get_sync_status(
        &self,
        index: &Entity,
        serialized_prefab: Option<&SerializedEntity>,
        should_have_prefab_entity: bool,
    ) -> SyncStatus;

    /// `create_yaml_component` creates a YamlValue out of our Component,
    /// ready for serialization. If succesful, the Value will be a Mapping of a
    /// SerializedComponent<T>, else, it will be a Value::Null.
    fn create_yaml_component(&self, index: &Entity) -> serde_yaml::Value;

    /// Given a SerializedEntity, this function will find the correct YamlValue inside it
    /// and return it, nice and easy!
    fn get_yaml_component(&self, serialized_entity: &SerializedEntity) -> serde_yaml::Value;
    fn get_yaml_component_key(&self) -> serde_yaml::Value;
}

impl<T> ComponentListBounds for ComponentList<T>
where
    T: ComponentBounds + ComponentPostDeserialization + SerializableComponent,
{
    fn expand_list(&mut self) {
        self.expand_list();
    }

    fn unset_component(&mut self, index: &Entity, scene_graph: &SceneGraph) -> bool {
        self.unset_component(index, scene_graph)
    }

    fn dump_to_log(&self, index: &Entity) {
        let comp_name = super::imgui_system::typed_text_ui::<T>();
        if let Some(comp) = self.get(index) {
            println!("{}: {:#?}", comp_name, comp);
        } else {
            println!("{}: None", comp_name);
        }
    }

    fn clone_entity(&mut self, original: &Entity, new_entity: &Entity, scene_graph: &mut SceneGraph) {
        if self.get(original).is_some() {
            let new_component = self.get(original).unwrap().inner().clone();
            self.set_component(new_entity, new_component, scene_graph);
        }
    }
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>, scene_graph: &mut SceneGraph) {
        if imgui::MenuItem::new(&imgui::ImString::new(super::imgui_system::typed_text_ui::<T>()))
            .enabled(self.get(index).is_none())
            .build(ui)
        {
            self.set_component_default(index, scene_graph);
        }
    }

    fn component_inspector(
        &mut self,
        entity: &Entity,
        scene_mode: SceneMode,
        prefab_sync_status: SyncStatus,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &Ui<'_>,
        is_open: bool,
    ) -> (Option<ComponentSerializationCommandType>, bool) {
        if let Some(comp) = self.get_mut(entity) {
            super::imgui_system::component_inspector_raw(
                comp,
                scene_mode,
                prefab_sync_status,
                entities,
                entity_names,
                prefab_hashmap,
                ui,
                is_open,
                true,
                |inner, ip| inner.entity_inspector(ip),
            )
        } else {
            (None, false)
        }
    }

    // fn serialization_option(
    //     &self,
    //     ui: &imgui::Ui<'_>,
    //     entity_id: &Entity,
    //     serialized_markers: &EntitySerializationMap,
    // ) -> Option<ComponentSerializationCommandType> {
    //     self.serialization_option_raw(ui, entity_id, serialized_markers)
    // }

    fn load_component_into_serialized_entity(
        &self,
        entity: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &TrackedEntitiesMap,
    ) {
        if let Some(member_component) = self.get(entity) {
            if member_component
                .inner()
                .is_serialized(serialized_entity, member_component.is_active)
                == false
            {
                member_component.inner().commit_to_scene(
                    serialized_entity,
                    member_component.is_active,
                    serialization_markers,
                );
            }
        }
    }

    fn post_deserialization(&mut self, entity_serde: &TrackedEntitiesMap, scene_graph: &SceneGraph) {
        for this_one in self.iter_mut() {
            let id = this_one.entity_id();
            this_one
                .inner_mut()
                .post_deserialization(id, entity_serde, scene_graph);
        }
    }

    fn get_for_post_deserialization(
        &mut self,
        index: &Entity,
    ) -> Option<(&mut dyn ComponentPostDeserialization, bool)> {
        self.get_mut(index).map(|component| {
            let is_active = component.is_active;
            (component.inner_mut() as _, is_active)
        })
    }

    fn get_sync_status(
        &self,
        index: &Entity,
        serialized_prefab: Option<&SerializedEntity>,
        should_have_prefab_entity: bool,
    ) -> SyncStatus {
        self.get(index).map_or(SyncStatus::Unsynced, |cmp| {
            SyncStatus::new(cmp, serialized_prefab, should_have_prefab_entity)
        })
    }

    fn create_yaml_component(&self, index: &Entity) -> serde_yaml::Value {
        self.get(index)
            .map(|comp| SerializedEntity::create_yaml_component(comp))
            .unwrap_or_default()
    }

    fn get_yaml_component(&self, serialized_entity: &SerializedEntity) -> serde_yaml::Value {
        SerializedEntity::get_serialized_yaml_component::<T>(serialized_entity)
    }

    fn get_yaml_component_key(&self) -> serde_yaml::Value {
        T::SERIALIZATION_NAME.clone()
    }
}
