use super::{
    imgui_component_utils::*, Component, ComponentList, Entity, Name, PrefabMap, SerializationMarker,
    SerializedEntity,
};
use imgui::Ui;

pub trait ComponentBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
    fn is_serialized(&self, serialized_entity: &SerializedEntity, active: bool) -> bool;
    fn commit_to_scene(
        &self,
        serialized_entity: &mut SerializedEntity,
        active: bool,
        serialization_marker: &super::ComponentList<super::SerializationMarker>,
    );
    fn uncommit_to_scene(&self, serialized_entity: &mut SerializedEntity);
    fn post_deserialization(&mut self, _: Entity, _: &ComponentList<SerializationMarker>) {}
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

pub struct InspectorParameters<'a, 'b> {
    pub ui: &'b imgui::Ui<'a>,
    pub entities: &'b [Entity],
    pub entity_names: &'b ComponentList<Name>,
    pub prefabs: &'b PrefabMap,
    pub uid: &'b str,
    pub is_open: bool,
}

pub trait ComponentListBounds {
    fn expand_list(&mut self);
    fn unset(&mut self, index: &Entity) -> bool;
    fn get_mut(&mut self, index: &Entity) -> Option<(&mut dyn ComponentBounds, bool)>;
    fn dump_to_log(&self, index: &Entity);
    fn clone_entity(&mut self, index: &Entity, new_entity: &Entity);

    // IMGUI
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>);
    #[must_use]
    fn component_inspector(
        &mut self,
        index: &Entity,
        parent_sync_status: Option<ParentSyncStatus>,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &imgui::Ui<'_>,
        is_open: bool,
    ) -> Option<ComponentSerializationCommandType>;

    #[must_use]
    fn serialization_option(
        &self,
        ui: &Ui<'_>,
        index: &Entity,
        serialized_marker: &ComponentList<super::SerializationMarker>,
    ) -> Option<ComponentSerializationCommandType>;

    fn load_component_into_serialized_entity(
        &self,
        index: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &ComponentList<super::SerializationMarker>,
    );

    fn post_deserialization(&mut self, entity_names: &ComponentList<SerializationMarker>);

    // This gets the sync status of a given Entity, provided, optionally, two serialized entities.
    // In the most general sense, the returned SyncStatus
    fn get_sync_status(
        &self,
        index: &Entity,
        serialized_entity: Option<&SerializedEntity>,
        serialized_prefab: Option<&SerializedEntity>,
        should_have_serialized_entity: bool,
        should_have_prefab_entity: bool,
    ) -> Option<ParentSyncStatus>;

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
    T: ComponentBounds + SerializableComponent,
{
    fn expand_list(&mut self) {
        self.expand_list();
    }

    fn unset(&mut self, index: &Entity) -> bool {
        self.unset(index)
    }

    fn dump_to_log(&self, index: &Entity) {
        let comp_name = super::imgui_system::typed_text_ui::<T>();
        if let Some(comp) = self.get(index) {
            println!("{}: {:#?}", comp_name, comp);
        } else {
            println!("{}: None", comp_name);
        }
    }

    fn clone_entity(&mut self, original: &Entity, new_entity: &Entity) {
        if self.get(original).is_some() {
            let new_component = self.get(original).unwrap().inner().clone();
            self.set_component(new_entity, new_component);
        }
    }
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>) {
        if imgui::MenuItem::new(&imgui::ImString::new(super::imgui_system::typed_text_ui::<T>()))
            .enabled(self.get(index).is_none())
            .build(ui)
        {
            self.set_component(index, T::default());
        }
    }

    fn component_inspector(
        &mut self,
        entity: &Entity,
        parent_sync_status: Option<ParentSyncStatus>,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &Ui<'_>,
        is_open: bool,
    ) -> Option<ComponentSerializationCommandType> {
        if let Some(comp) = self.get_mut(entity) {
            let ParentSyncStatus { serialized, prefab } = parent_sync_status.unwrap();

            let (serialization_command, delete) = super::imgui_system::component_inspector_raw(
                comp,
                serialized,
                prefab,
                entities,
                entity_names,
                prefab_hashmap,
                ui,
                is_open,
                true,
                |inner, ip| inner.entity_inspector(ip),
            );

            if delete {
                self.unset(entity);
            }

            serialization_command
        } else {
            None
        }
    }

    fn serialization_option(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        serialized_markers: &ComponentList<super::SerializationMarker>,
    ) -> Option<ComponentSerializationCommandType> {
        self.serialization_option_raw(ui, entity_id, serialized_markers)
    }

    fn load_component_into_serialized_entity(
        &self,
        entity: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &ComponentList<super::SerializationMarker>,
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

    fn post_deserialization(&mut self, entity_serde: &ComponentList<SerializationMarker>) {
        for this_one in self.iter_mut() {
            let id = this_one.entity_id();
            this_one.inner_mut().post_deserialization(id, entity_serde);
        }
    }

    fn get_mut(&mut self, index: &Entity) -> Option<(&mut dyn ComponentBounds, bool)> {
        self.get_mut(index).map(|component| {
            let is_active = component.is_active;
            (component.inner_mut() as _, is_active)
        })
    }

    fn get_sync_status(
        &self,
        index: &Entity,
        serialized_entity: Option<&SerializedEntity>,
        serialized_prefab: Option<&SerializedEntity>,
        should_have_serialized_entity: bool,
        should_have_prefab_entity: bool,
    ) -> Option<ParentSyncStatus> {
        self.get(index).map(|cmp| {
            ParentSyncStatus::new(
                cmp,
                serialized_entity,
                serialized_prefab,
                should_have_serialized_entity,
                should_have_prefab_entity,
            )
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

impl<T> ComponentList<T>
where
    T: ComponentBounds + Default + Clone + typename::TypeName + 'static,
{
    /// Simply a wrapper around creating a new component
    pub fn set_component(&mut self, entity_id: &Entity, new_component: T) {
        self.set(&entity_id, Component::new(&entity_id, new_component));
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component_with_active(&mut self, entity_id: &Entity, new_component: T, active: bool) {
        self.set(
            &entity_id,
            Component::with_active(&entity_id, new_component, active),
        );
    }

    /// Gets a mutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// a mutable reference to that.
    pub fn get_mut_or_default(&mut self, index: &Entity) -> &mut Component<T> {
        if self.get_mut(index).is_none() {
            self.set_component(index, T::default());
        }

        self.get_mut(index).unwrap()
    }

    /// Gets an immutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// an immutable reference to that. This is **slower** than just
    /// `get`, so use that if you can help it.
    pub fn get_or_default(&mut self, index: &Entity) -> &Component<T> {
        if self.get(index).is_none() {
            self.set_component(index, T::default());
        }

        self.get(index).unwrap()
    }
}
