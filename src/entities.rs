mod generational_index;
mod generational_index_array;
mod generational_index_value;

pub use generational_index::*;
pub use generational_index_array::*;
pub use generational_index_value::*;

pub type Entity = generational_index::GenerationalIndex;
pub type EntityAllocator = GenerationalIndexAllocator;
pub type ComponentList<T> = generational_index_array::GenerationalIndexArray<super::Component<T>>;
