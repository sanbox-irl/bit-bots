use super::{GenerationalIndex, GenerationalIndexValue};

#[derive(Serialize, Deserialize, Default, Debug)]
struct ArrayEntry<T: GenerationalIndexValue> {
    value: T,
    generation: u64,
}

// An array from GenerationalIndex to some Value T.
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationalIndexArray<T: GenerationalIndexValue>(Vec<Option<ArrayEntry<T>>>);

impl<T: GenerationalIndexValue> GenerationalIndexArray<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns a mutable Iterator over the internal members of the Vec.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut().flat_map(|opt_ent| {
            let value = &mut opt_ent.as_mut()?.value;
            if value.is_active() {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Returns an immutable Iterator over the internal members of the Vec.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|opt_ent| {
            let value = &opt_ent.as_ref()?.value;
            if value.is_active() {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Set the value for some generational index.  May overwrite past generation
    /// values.
    pub fn set(&mut self, index: &GenerationalIndex, value: T) {
        self.0[index.index] = Some(ArrayEntry {
            value,
            generation: index.generation,
        });
    }

    /// Adds a new component for a new entity to the end of the List. Don't use this
    /// very commonly -- this is for adding new entities, essentially.
    pub fn expand_list(&mut self) {
        self.0.push(None);
    }

    /// Unsets the value for some generational index. Returns true if succesfully
    /// unset.
    pub fn unset(&mut self, index: &GenerationalIndex) -> bool {
        let ret = &self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                self.0[index.index] = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Checks if the desired index points to a valid location. Merely shorthand
    // for doing a .is_some() after a get.
    pub fn contains(&self, index: &GenerationalIndex) -> bool {
        self.get(index).is_some()
    }

    // Gets an immutable reference to the contained value, if it exists.
    pub fn get(&self, index: &GenerationalIndex) -> Option<&T> {
        let ret = &self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                Some(&ret.value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets a mutable reference to the contained value, if it exists.
    pub fn get_mut(&mut self, index: &GenerationalIndex) -> Option<&mut T> {
        let ret = &mut self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                Some(&mut ret.value)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T: GenerationalIndexValue> Default for GenerationalIndexArray<T> {
    fn default() -> Self {
        Self::new()
    }
}
