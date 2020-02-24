pub use super::*;

mod fragmented_data;
pub mod serialization_util;
mod serialized_entity;

#[cfg(debug_assertions)]
pub mod update_serialization;

pub use fragmented_data::FragmentedData;
pub use serialized_entity::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SerializationDelta {
    Unchanged,
    Updated,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[must_use]
pub struct PostDeserializationRequired;

impl PostDeserializationRequired {
    pub fn fold_in(&self, _: PostDeserializationRequired) {}
}
