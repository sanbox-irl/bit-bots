use super::*;

use anyhow::{Error, Result as AnyResult};
use std::fs;

mod serialize_entities;
pub mod entities {
    pub use super::serialize_entities::*;
}

mod serialize_game_config;
pub mod game_config {
    pub use super::serialize_game_config::*;
}

mod serialize_prefabs;
pub mod prefabs {
    pub use super::serialize_prefabs::*;
}

mod serialize_singleton_components;
pub mod singleton_components {
    pub use super::serialize_singleton_components::*;
}

mod serialize_sprites;
pub mod sprites {
    pub use super::serialize_sprites::*;
}

mod serialize_tilemaps;
pub mod tilemaps {
    pub use super::serialize_tilemaps::*;
}

mod serialize_tilesets;
pub mod tilesets {
    pub use super::serialize_tilesets::*;
}

pub(super) fn load_serialized_file<T: Default>(path: &str) -> Result<T, Error>
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

pub fn save_serialized_file<T>(item: &T, path: &str) -> Result<(), Error>
where
    T: serde::Serialize,
{
    let s = serde_yaml::to_string(item)?;
    Ok(fs::write(path, s)?)
}

// fn load_file_bin<T: Default>(path: &str) -> Result<T, Error>
// where
//     for<'de> T: serde::Deserialize<'de>,
// {
//     let path = std::path::Path::new(path);
//     fs::create_dir_all(path.parent().unwrap())?;

//     if path.exists() == false {
//         fs::File::create(path)?;
//     }

//     let file_bits: Vec<u8> = fs::read(path)?;
//     Ok(bincode::deserialize(&file_bits).unwrap_or_default())
// }

// fn save_file_bin<T>(item: &T, path: &str) -> Result<(), Error>
// where
//     T: serde::Serialize,
// {
//     let path = std::path::Path::new(path);
//     fs::create_dir_all(path.parent().unwrap())?;

//     if path.exists() == false {
//         fs::File::create(path)?;
//     }

//     let s = bincode::serialize(item)?;
//     Ok(fs::write(path, s)?)
// }
