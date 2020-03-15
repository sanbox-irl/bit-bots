use super::PrefabId;

pub const SCENE_DIRECTORY: &str = "assets/serialized_data/scenes";
pub const PREFAB_DIRECTORY: &str = "assets/serialized_data/prefabs";
pub const DEFAULT_SINGLETONS_SUBPATH: &str = "default_singleton_data.yaml";

pub const TILEMAP_SUBPATH: &str = "tilemap";
pub const SCENE_SUBPATH: &str = "scene_graph.graph";
pub const ENTITY_SUBPATH: &str = "entities_data.yaml";
pub const SINGLETONS_SUBPATH: &str = "singleton_data.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    name: String,
    is_prefab: bool,
    #[serde(skip)]
    mode: SceneMode,
}

impl Scene {
    pub const fn new(name: String) -> Self {
        Scene {
            name,
            is_prefab: false,
            mode: SceneMode::Draft,
        }
    }

    pub fn new_prefab(prefab_id: PrefabId) -> Self {
        Scene {
            name: prefab_id.inner().to_string(),
            is_prefab: true,
            mode: SceneMode::Draft,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_prefab(&self) -> bool {
        self.is_prefab
    }

    pub fn mode(&self) -> SceneMode {
        self.mode
    }

    pub fn play_scene(&mut self) {
        self.mode = SceneMode::Playing;
    }

    pub fn pause_scene(&mut self) {
        self.mode = SceneMode::Paused;
    }

    pub fn entity_path(&self) -> String {
        if self.is_prefab {
            format!("{}/{}.prefab", PREFAB_DIRECTORY, &self.name)
        } else {
            format!("{}/{}/{}", SCENE_DIRECTORY, &self.name, ENTITY_SUBPATH)
        }
    }

    pub fn singleton_path(&self) -> String {
        if self.is_prefab {
            format!("{}/{}", PREFAB_DIRECTORY, DEFAULT_SINGLETONS_SUBPATH)
        } else {
            format!("{}/{}/{}", SCENE_DIRECTORY, &self.name, SINGLETONS_SUBPATH)
        }
    }

    pub fn tilemap_path(&self, tilemap_path_end: &str) -> String {
        if self.is_prefab {
            format!("{}/{}/{}", PREFAB_DIRECTORY, TILEMAP_SUBPATH, tilemap_path_end)
        } else {
            format!(
                "{}/{}/{}/{}",
                SCENE_DIRECTORY, &self.name, TILEMAP_SUBPATH, tilemap_path_end
            )
        }
    }

    pub fn scene_graph_path(&self) -> String {
        if self.is_prefab {
            // loads the prefab
            self.entity_path()
        } else {
            format!("{}/{}/{}", SCENE_DIRECTORY, &self.name, SCENE_SUBPATH)
        }
    }
}

use std::fmt;

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            if self.is_prefab { "Prefab" } else { "Scene" },
            self.name
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SceneMode {
    Draft,
    Playing,
    Paused,
}

impl Default for SceneMode {
    fn default() -> Self {
        SceneMode::Draft
    }
}

pub struct SceneIsDraft(());
impl SceneIsDraft {
    pub fn new(scene_mode: SceneMode) -> Option<SceneIsDraft> {
        match scene_mode {
            SceneMode::Draft => Some(SceneIsDraft(())),
            SceneMode::Playing => None,
            SceneMode::Paused => None,
        }
    }
}