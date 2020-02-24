use super::*;
use clockwork_build_shared::sprite_packing;
use glob::glob;
use log::info;
use serialization_util::SerializedMetaData;
use sheep::InputSprite;
use std::collections::HashMap;
use std::fs;

const GEN_DIRECTORY: &str = "assets/gen/textures";
const MANIFEST: &str = "assets/gen/textures/manifest.yaml";
const PNG_GLOB: &str = "assets/textures/**/*.png";

pub fn pack_sprites() -> Result<(), Error> {
    initialize_directory()?;

    let mut manifest: HashMap<String, SerializedMetaData> =
        serialization_util::load(MANIFEST).unwrap_or_default();

    let pack_sprite_info = {
        let mut filenames = Vec::new();
        let mut input_sprites = Vec::new();

        let mut repack_textures = false;

        for entry in glob(PNG_GLOB)? {
            let entry = entry?;
            let filename = entry.file_stem().unwrap().to_str().unwrap().to_owned();
            let current_metadata = fs::metadata(entry)?;

            match manifest.get(&filename) {
                Some(ser_metadata) => {
                    if current_metadata.created()? != ser_metadata.created
                        || current_metadata.modified()? != ser_metadata.modified
                    {
                        info!("{} has been changed.", filename);
                        repack_textures = true;
                        manifest.insert(filename, SerializedMetaData::new(current_metadata)?);
                    }
                }

                None => {
                    info!("{} was not present in manifest.", filename);
                    repack_textures = true;
                    manifest.insert(filename, SerializedMetaData::new(current_metadata)?);
                }
            }
        }

        if repack_textures {
            for entry in glob(PNG_GLOB)? {
                let entry = entry?;
                let filename = entry.file_stem().unwrap().to_str().unwrap().to_owned();
                filenames.push(filename);

                let image = image::open(entry)?;
                let image = image.as_rgba8().expect("Couldn't construct the image!");

                let dimensions = (image.width(), image.height());
                let bytes = image.pixels().flat_map(|it| it.0.iter().map(|it| *it)).collect();

                input_sprites.push(InputSprite { bytes, dimensions });
            }
            Some((input_sprites, filenames))
        } else {
            None
        }
    };

    match pack_sprite_info {
        Some((input_sprites, file_names)) => {
            info!("Repacking sprites...");
            serialization_util::save(&manifest, MANIFEST)?;
            sprite_packing::parse_sprites::pack_sprites(input_sprites, file_names)
        }
        None => {
            info!("Sprites were not repacked.");
            Ok(())
        }
    }
}

fn initialize_directory() -> Result<(), Error> {
    fs::create_dir_all(GEN_DIRECTORY)?;
    Ok(())
}
