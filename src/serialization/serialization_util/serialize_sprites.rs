use super::sprite_resources::*;
use super::*;
use clockwork_build_shared::sprite_packing::shared::SpriteSheet;

const SPRITE_PATH: &str = "assets/serialized_data/resources/sprite_data.yaml";

pub fn serialize_sprite(sprite_data: &SpriteData) -> Result<(), Error> {
    // LOAD EXISTING SPRITE DATA VEC
    let mut serialized_sprites: Vec<SpriteInGameData> = load_serialized_file(SPRITE_PATH)?;

    // CHECK IF THIS SPRITE_DATA IS IN SPRITE
    if let Some(pos) = serialized_sprites
        .iter()
        .position(|serialized_sprite| serialized_sprite.sprite_name == sprite_data.sprite_name)
    {
        serialized_sprites[pos] = sprite_data.clone().into();
    } else {
        serialized_sprites.push(sprite_data.clone().into());
    }

    // RESAVE SPRITE DATA VEC
    save_serialized_file(&serialized_sprites, SPRITE_PATH)?;

    Ok(())
}

pub fn load_sprite(sprite_name: SpriteName, texture_page_handle: usize) -> Result<SpriteData, Error> {
    let sprites: Vec<SpriteInGameData> = load_sprites()?;
    let sprite_sheet = load_spritesheets()?;

    let sprite_sheet_data = sprite_sheet
        .sprites
        .into_iter()
        .find(|s| s.name == sprite_name.to_string())
        .unwrap();

    let sprite_ingamedata = sprites
        .into_iter()
        .find(|s| s.sprite_name == sprite_name)
        .unwrap_or_else(|| SpriteInGameData::create_default(&sprite_sheet_data, sprite_name));

    Ok(SpriteData::from_sprite_resource(
        sprite_sheet_data,
        sprite_ingamedata,
        sprite_name,
        TextureInformation {
            page: texture_page_handle,
            dimensions: Vec2::new(
                sprite_sheet.texture_width as f32,
                sprite_sheet.texture_height as f32,
            ),
        },
    ))
}

pub fn load_sprites() -> Result<Vec<SpriteInGameData>, Error> {
    let sprites: Vec<SpriteInGameData> = load_serialized_file(SPRITE_PATH)?;
    Ok(sprites)
}

pub fn load_spritesheets() -> Result<SpriteSheet, Error> {
    load_serialized_file("assets/gen/textures/packed_sheet_0.yaml")
}
