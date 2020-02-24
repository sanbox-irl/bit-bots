use super::{ComponentBounds, Entity, GenerationalIndexValue};

#[derive(Debug)]
pub struct Component<T: ComponentBounds + Clone> {
    pub is_active: bool,
    entity_id: Entity,
    inner: T,
}

impl<T: ComponentBounds + Clone> Component<T> {
    pub fn new(entity_id: &Entity, inner: T) -> Self {
        Self {
            entity_id: *entity_id,
            inner,
            is_active: true,
        }
    }

    pub fn with_active(entity_id: &Entity, inner: T, is_active: bool) -> Self {
        Self {
            entity_id: *entity_id,
            inner,
            is_active,
        }
    }

    pub fn entity_id(&self) -> Entity {
        self.entity_id
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn clone_inner(&self) -> T {
        self.inner.clone()
    }

    pub fn fast_serialize(&self) -> Option<(T, bool)> {
        Some((self.inner.clone(), self.is_active))
    }

    pub fn is_serialized(&self, serialized_entity: &super::SerializedEntity) -> bool {
        self.inner.is_serialized(serialized_entity, self.is_active)
    }
}

impl<T: ComponentBounds + Clone> GenerationalIndexValue for Component<T> {
    fn is_active(&self) -> bool {
        self.is_active
    }
}

use std::fmt::{self, Display};
impl<T: Display + ComponentBounds + Clone> Display for Component<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID: {}, Inner: {}", self.entity_id, self.inner)
    }
}

/// This shouldn't really be used -- it's actually just a glue,
/// since we occasionally pass around (&T<ComponentBounds>, is_active)
/// and Component itself, which are in a sense, the same thing.
/// This allows us to `.into` them together
pub struct ComponentData<'a, T: ComponentBounds> {
    entity_id: Entity,
    inner_mut: &'a mut T,
}

impl<'a, T: ComponentBounds> ComponentData<'a, T> {
    pub fn entity_id(&self) -> Entity {
        self.entity_id
    }

    pub fn inner_mut(&mut self) -> &mut T {
        self.inner_mut
    }
}

impl<'a, T: ComponentBounds + Clone> From<&'a mut Component<T>> for ComponentData<'a, T> {
    fn from(other: &'a mut Component<T>) -> Self {
        ComponentData {
            entity_id: other.entity_id(),
            inner_mut: other.inner_mut(),
        }
    }
}

impl<'a, T: ComponentBounds + Clone> From<(&'a mut T, Entity)> for ComponentData<'a, T> {
    fn from(other: (&'a mut T, Entity)) -> Self {
        ComponentData {
            entity_id: other.1,
            inner_mut: other.0,
        }
    }
}
