mod generational_index;
mod generational_index_array;
mod generational_index_value;

pub use generational_index::*;
pub use generational_index_array::*;
pub use generational_index_value::*;

pub type Entity = generational_index::GenerationalIndex;
pub type EntityAllocator = GenerationalIndexAllocator;
pub type ComponentList<T> = generational_index_array::GenerationalIndexArray<super::Component<T>>;

use super::{Component, ComponentBounds, NonSceneGraphComponent};

impl<T> ComponentList<T>
where
    T: ComponentBounds + Default + Clone + typename::TypeName + 'static,
{
    /// Wrapper around
    pub fn unset_component(&mut self, index: &Entity) -> bool {
        self.unset(index)
    }
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + NonSceneGraphComponent + Clone + Default + typename::TypeName + 'static,
{
    /// Simply a wrapper around creating a new component
    pub fn set_component_default(&mut self, entity_id: &Entity) -> &mut Component<T> {
        self.set(&entity_id, Component::new(&entity_id, T::default()));
        self.get_mut(&entity_id).unwrap()
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component(&mut self, entity_id: &Entity, new_component: T) -> &mut Component<T> {
        self.set(&entity_id, Component::new(&entity_id, new_component));
        self.get_mut(&entity_id).unwrap()
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
