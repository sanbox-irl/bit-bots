use super::{
    serialization_util, Camera, Entity, Marker, RenderingUtility, ResourcesDatabase, SingletonBounds,
    SingletonComponent,
};
use anyhow::{Error, Result as AnyResult};
use std::collections::HashMap;

pub type AssociatedEntityMap = HashMap<Marker, Entity>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SingletonDatabase {
    pub camera: SingletonComponent<Camera>,
    #[serde(skip)]
    pub rendering_utility: RenderingUtility,
    #[serde(skip)]
    pub associated_entities: AssociatedEntityMap,
}

impl SingletonDatabase {
    pub fn new(marker_map: AssociatedEntityMap) -> Result<SingletonDatabase, Error> {
        let mut serialized_singletons: SingletonDatabase =
            serialization_util::singleton_components::load_singleton_database()?;

        serialized_singletons.associated_entities = marker_map;
        Ok(serialized_singletons)
    }

    pub fn save_singleton_markers(&self, entity: &Entity) -> Option<Marker> {
        for (this_marker, this_entity) in &self.associated_entities {
            if this_entity == entity {
                return Some(*this_marker);
            }
        }

        None
    }

    pub fn edit_serialized_singleton_database<T: SingletonBounds, F>(
        live_component: &mut SingletonComponent<T>,
        edit_function: F,
    ) -> AnyResult<()>
    where
        F: Fn(&mut SingletonDatabase, &mut SingletonComponent<T>),
    {
        let mut serialized_singletons: SingletonDatabase =
            serialization_util::singleton_components::load_singleton_database()?;

        edit_function(&mut serialized_singletons, live_component);

        serialization_util::singleton_components::serialize_singleton_database(&serialized_singletons)
    }

    pub fn initialize_with_runtime_resources(
        &mut self,
        resources: &ResourcesDatabase,
        hwi: &super::HardwareInterface,
    ) {
        self.rendering_utility.initialize(resources);
        self.camera.inner_mut().initialize_with_hwi(hwi);
    }
}

impl Default for SingletonDatabase {
    fn default() -> Self {
        SingletonDatabase {
            // @update_singletons
            camera: SingletonComponent::new(Marker::Camera, Camera::default()),
            rendering_utility: RenderingUtility::default(),
            associated_entities: HashMap::new(),
        }
    }
}
