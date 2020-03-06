mod generational_index;
mod generational_index_array;
mod generational_index_value;

pub use generational_index::*;
pub use generational_index_array::*;
pub use generational_index_value::*;

pub type Entity = generational_index::GenerationalIndex;
pub type EntityAllocator = GenerationalIndexAllocator;
pub type ComponentList<T> = generational_index_array::GenerationalIndexArray<super::Component<T>>;

use super::{
    scene_graph::SceneGraph, Component, ComponentBounds, ComponentPostDeserialization, SerializableComponent,
};

impl<T> ComponentList<T>
where
    T: ComponentBounds
        + ComponentPostDeserialization
        + SerializableComponent
        + Clone
        + Default
        + typename::TypeName
        + 'static,
{
    /// Simply a wrapper around creating a new component
    pub fn set_component_default(
        &mut self,
        entity_id: &Entity,
        scene_graph: &mut SceneGraph,
    ) -> &mut Component<T> {
        self.core_set(entity_id, true, T::default(), scene_graph)
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component(
        &mut self,
        entity_id: &Entity,
        new_component: T,
        scene_graph: &mut SceneGraph,
    ) -> &mut Component<T> {
        self.core_set(entity_id, true, new_component, scene_graph)
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component_with_active(
        &mut self,
        entity_id: &Entity,
        active: bool,
        new_component: T,
        scene_graph: &mut SceneGraph,
    ) -> &mut Component<T> {
        self.core_set(entity_id, active, new_component, scene_graph)
    }

    fn core_set(
        &mut self,
        entity_id: &Entity,
        active: bool,
        new_component: T,
        scene_graph: &mut SceneGraph,
    ) -> &mut Component<T> {
        self.set(
            entity_id,
            Component::with_active(&entity_id, new_component, active),
        );
        let recent_component = self.get_mut(entity_id).unwrap();
        recent_component.inner_mut().on_set(entity_id, scene_graph);
        recent_component
    }

    /// Gets a mutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// a mutable reference to that.
    pub fn get_mut_or_default(&mut self, index: &Entity, scene_graph: &mut SceneGraph) -> &mut Component<T> {
        if self.get_mut(index).is_none() {
            self.set_component_default(index, scene_graph);
        }

        self.get_mut(index).unwrap()
    }

    /// Gets an immutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// an immutable reference to that. This is **slower** than just
    /// `get`, so use that if you can help it.
    pub fn get_or_default(&mut self, index: &Entity, scene_graph: &mut SceneGraph) -> &Component<T> {
        if self.get(index).is_none() {
            self.set_component_default(index, scene_graph);
        }

        self.get(index).unwrap()
    }

    /// Wrapper around
    pub fn unset_component(&mut self, index: &Entity, scene_graph: &mut SceneGraph) -> bool {
        let old = self.unset(index);
        if let Some(mut old) = old {
            let id = old.entity_id();
            old.inner_mut().on_unset(id, self, scene_graph);

            true
        } else {
            false
        }
    }
}
