use super::{InspectorParameters, Marker};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingletonComponent<T: SingletonBounds> {
    marker: Marker,
    inner: T,
}

impl<T: SingletonBounds> SingletonComponent<T> {
    pub fn new(marker: Marker, inner: T) -> Self {
        Self { marker, inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn marker(&self) -> Marker {
        self.marker
    }
}

pub trait SingletonBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
}
