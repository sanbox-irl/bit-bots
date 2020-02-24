use super::*;

pub fn path() -> String {
    scene_system::CURRENT_SCENE.lock().unwrap().singleton_path()
}

pub fn load_singleton_database() -> Result<SingletonDatabase, Error> {
    load_serialized_file(&path())
}

pub fn serialize_singleton_database(singleton_database: &SingletonDatabase) -> Result<(), Error> {
    save_serialized_file(singleton_database, &path())
}
