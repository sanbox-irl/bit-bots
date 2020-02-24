use super::*;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref PREFAB_GLOB: String = format!("{}/**/*.prefab", PREFAB_DIRECTORY);
}

fn path(entity_id: &str) -> String {
    format!("{}/{}.prefab", PREFAB_DIRECTORY, entity_id)
}

fn invalid_path(entity_id: &str) -> String {
    format!("{}/invalid_prefabs/{}.prefab", PREFAB_DIRECTORY, entity_id)
}

/// This is a weird function. We essentially are going to pass the prefab
/// through Serde. We occasionally (by making some things Serde ignore),
/// manipulate runtime data to be invisible to Serde, but this can make
/// live prefab instantiation create incorrect data. If you cycle a prefab
/// through this function, the prefab returned will be stripped of all
/// runtime data, as if it was loaded off disk.
pub fn cycle_prefab(prefab: Prefab) -> Result<Prefab, Error> {
    Ok(serde_yaml::from_value(serde_yaml::to_value(prefab)?)?)
}

pub fn serialize_prefab(prefab: &Prefab) -> Result<(), Error> {
    let path = path(&prefab.root_id().to_string());

    save_serialized_file(&prefab, &path)
}

pub fn invalidate_prefab(prefab: &Prefab) -> AnyResult<()> {
    let path = path(&prefab.root_id().to_string());
    fs::remove_file(&path)?;

    save_serialized_file(prefab, &invalid_path(&prefab.root_id().to_string()))
}

pub fn load_prefab(prefab_id: &Uuid) -> Result<Option<Prefab>, Error> {
    // ENTITIES
    let prefab: Result<Prefab, _> = load_serialized_file(&path(&prefab_id.to_string()));

    Ok(prefab
        .map_err(|e| error!("Error loading Prefab File: {}", e))
        .map(|ok| {
            assert_eq!(&ok.root_id(), prefab_id);
            ok
        })
        .ok())
}

pub fn load_all_prefabs() -> AnyResult<PrefabMap> {
    let mut ret = std::collections::HashMap::new();

    for path in glob::glob(&PREFAB_GLOB)? {
        let path = path?;
        let prefab: Prefab = load_serialized_file(path.to_str().unwrap())?;
        ret.insert(prefab.root_id(), prefab);
    }

    Ok(ret)
}
