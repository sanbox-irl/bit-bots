use super::{Camera, Entity, Marker, RenderingUtility, ResourcesDatabase, SingletonComponent};
use std::collections::HashMap;

pub type AssociatedEntityMap = HashMap<Marker, Entity>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingletonDatabase {
    pub camera: SingletonComponent<Camera>,
    #[serde(skip)]
    pub rendering_utility: RenderingUtility,
    #[serde(skip)]
    pub associated_entities: AssociatedEntityMap,
}

impl SingletonDatabase {
    pub fn save_singleton_markers(&self, entity: &Entity) -> Option<Marker> {
        for (this_marker, this_entity) in &self.associated_entities {
            if this_entity == entity {
                return Some(*this_marker);
            }
        }

        None
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
