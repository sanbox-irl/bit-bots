use super::{
    serialization_util, Scene, SceneMode, SerializedEntity, SingletonDatabase, ENTITY_SUBPATH,
    PREFAB_DIRECTORY, SCENE_DIRECTORY, SINGLETONS_SUBPATH,
};
use anyhow::Error;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref CURRENT_SCENE: Mutex<Scene> = Mutex::new(Scene::new("NULL".to_string()));
    pub static ref NEXT_SCENE: Mutex<Option<Scene>> = Mutex::new(Some(Scene::new("1".to_string())));
}

pub fn current_scene_mode() -> SceneMode {
    CURRENT_SCENE.lock().unwrap().mode()
}

pub fn current_scene_name() -> String {
    CURRENT_SCENE.lock().unwrap().name().to_owned()
}

pub fn set_next_scene(scene: Scene) -> bool {
    if scene_exists(&scene) == false {
        return false;
    }

    let mut next_scene_handle = NEXT_SCENE.lock().unwrap();
    *next_scene_handle = Some(scene);

    true
}

pub fn create_scene(scene_name: &str) -> Result<bool, Error> {
    let scene = Scene::new(scene_name.to_string());

    if scene_exists(&scene) {
        return Ok(false);
    }

    // Create the Scene Folder
    let scene_path = format!("{}/{}", SCENE_DIRECTORY, scene_name);
    std::fs::create_dir_all(&scene_path)?;

    // Entities Data
    {
        let blank_entity_save_data: Vec<SerializedEntity> = vec![];
        let entity_path = format!("{}/{}", scene_path, ENTITY_SUBPATH);
        serialization_util::save_serialized_file(&blank_entity_save_data, &entity_path)?;
    }

    // Make a blank singleton database!
    {
        let singleton_database_blank: SingletonDatabase = SingletonDatabase::default();
        let singleton_path = format!("{}/{}", scene_path, SINGLETONS_SUBPATH);
        serialization_util::save_serialized_file(&singleton_database_blank, &singleton_path)?;
    }

    Ok(true)
}

pub fn delete_scene(name: &str) -> Result<bool, Error> {
    let scene = Scene::new(name.to_string());

    if scene_exists(&scene) == false {
        return Ok(false);
    }

    let path = format!("{}/{}", SCENE_DIRECTORY, name);
    std::fs::remove_dir_all(&path)?;

    Ok(true)
}

fn scene_exists(scene: &Scene) -> bool {
    let path = if scene.is_prefab() {
        format!("{}/{}.prefab", PREFAB_DIRECTORY, scene.name())
    } else {
        format!("{}/{}", SCENE_DIRECTORY, scene.name())
    };

    let path = std::path::Path::new(&path);
    path.exists()
}
