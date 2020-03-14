use super::{
    serialization_util, Scene, SerializedEntity, SingletonDatabase, ENTITY_SUBPATH, PREFAB_DIRECTORY,
    SCENE_DIRECTORY, SINGLETONS_SUBPATH,
};
use anyhow::Error;

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

pub fn scene_exists(scene: &Scene) -> bool {
    let path = if scene.is_prefab() {
        format!("{}/{}.prefab", PREFAB_DIRECTORY, scene.name())
    } else {
        format!("{}/{}", SCENE_DIRECTORY, scene.name())
    };

    let path = std::path::Path::new(&path);
    path.exists()
}
