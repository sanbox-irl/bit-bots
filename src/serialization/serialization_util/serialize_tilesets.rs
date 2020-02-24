use super::{tile_resources::*, *};

const TILESET_PATH: &str = "assets/serialized_data/resources/tileset_data.yaml";

pub fn serialize_tileset(new_tset: TileSet) -> Result<(), Error> {
    let mut serialized_tileset: Vec<TileSetSerialized> = load_serialized_tilesets()?;
    if let Some(pos) = serialized_tileset
        .iter()
        .position(|tset| tset.name == new_tset.name)
    {
        serialized_tileset[pos] = new_tset.into();
    } else {
        serialized_tileset.push(new_tset.into());
    }
    serialize_all_tilesets(&serialized_tileset)
}

pub fn serialize_all_tilesets(tsets: &[TileSetSerialized]) -> Result<(), Error> {
    save_serialized_file(&tsets, TILESET_PATH)
}

pub fn load_tileset(tileset_name: TileSetName) -> Result<Option<TileSet>, Error> {
    let mut all_serialized_tsets = load_serialized_tilesets()?;

    if let Some(serialized_tset) = all_serialized_tsets.iter().find(|i| i.name == tileset_name) {
        Ok(Some(serialized_tset.clone().into()))
    } else {
        error!(
            "There is no serialized version of the tileset by name {}.",
            tileset_name
        );
        error!("Serializing default data, but not overwriting current tileset.");
        error!("If you revert again, we'll go back to default!");

        let new_tileset = TileSet {
            name: tileset_name,
            ..TileSet::default()
        };

        all_serialized_tsets.push(new_tileset.into());
        serialize_all_tilesets(&all_serialized_tsets)?;

        Ok(None)
    }
}

pub fn load_all_tilesets() -> Result<Vec<TileSet>, Error> {
    let tile_set_serialized: Vec<TileSetSerialized> = load_serialized_tilesets()?;

    Ok(tile_set_serialized.iter().map(|i| i.clone().into()).collect())
}

pub fn load_serialized_tilesets() -> Result<Vec<TileSetSerialized>, Error> {
    load_serialized_file(TILESET_PATH)
}
