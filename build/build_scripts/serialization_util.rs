use super::Error;
use std::fs;
use std::time::SystemTime;

pub fn load<T: Default>(path: &str) -> Result<T, Error>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let path = std::path::Path::new(path);
    fs::create_dir_all(path.parent().unwrap())?;

    if path.exists() == false {
        fs::File::create(path)?;
    }

    let file_string = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&file_string).unwrap_or_default())
}

pub fn save<T>(item: &T, path: &str) -> Result<(), Error>
where
    T: serde::Serialize,
{
    let s = serde_yaml::to_string(item)?;
    Ok(fs::write(path, s)?)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SerializedMetaData {
    pub created: SystemTime,
    pub modified: SystemTime,
}

impl SerializedMetaData {
    pub fn new(metadata: fs::Metadata) -> Result<SerializedMetaData, Error> {
        Ok(SerializedMetaData {
            created: metadata.created()?,
            modified: metadata.modified()?,
        })
    }
}
