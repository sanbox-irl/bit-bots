use super::{
    fonts::{FontData, FontName},
    game_config::Config,
    renderer_system, serialization_util,
    sprite_resources::{SpriteData, SpriteInGameData, SpriteName, TextureInformation},
    tile_resources::{TileSet, TileSetName, TileSetSerialized},
    Prefab, PrefabMap, RendererComponent, SoundResource, Vec2,
};
use anyhow::Error;
use glyph_brush::GlyphBrushBuilder;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub struct ResourcesDatabase {
    pub sprites: HashMap<SpriteName, SpriteData>,
    pub tilesets: HashMap<TileSetName, TileSet>,
    pub sounds: HashMap<SoundResource, Cursor<&'static [u8]>>,
    pub fonts: HashMap<FontName, FontData>,
    pub config: Config,
    prefabs: PrefabMap,
}

impl ResourcesDatabase {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
            sprites: HashMap::new(),
            sounds: HashMap::new(),
            fonts: HashMap::new(),
            prefabs: HashMap::new(),
            config: serialization_util::game_config::load_config().unwrap_or_default(),
        }
    }

    pub fn initialize(&mut self, renderer: &mut RendererComponent) -> Result<(), Error> {
        info!("Loading Resources...");

        // LOAD SPRITES
        info!("....................Loading Sprites");
        let sprite_resource: Vec<u8> = ResourcesDatabase::load_spritesheets()?;
        let image = image::load_from_memory_with_format(&sprite_resource, image::ImageFormat::PNG)?.to_rgba();
        let handle = renderer_system::register_texture(renderer, &image)?;

        // LOAD PREFABS
        info!("....................Loading Prefabs");
        self.prefabs = serialization_util::prefabs::load_all_prefabs()?;
        info!("...✔ Loaded Resources");

        // INITIALIZE OTHER RESOURCES
        info!("Initializing Resources...");

        info!(".........................Initializing Sprites");
        self.initialize_sprites(handle)?;

        info!(".........................Initializing Tile Sets");
        self.initialize_tilesets()?;

        info!(".........................Initializing Sounds");
        self.initialize_sounds()?;

        info!(".........................Initializing Fonts");
        self.initialize_fonts()?;

        info!("...✔ All Resources Initialized");
        Ok(())
    }

    fn initialize_sprites(&mut self, texture_page_handle: usize) -> Result<(), Error> {
        let serialized_sprites = serialization_util::sprites::load_sprites()?;
        // @techdebt We'll need to support multiple sprite sheets at some point in our life.
        // If things aren't showing up, then this is why! Should be simple, just a glob.
        let sprite_sheet = serialization_util::sprites::load_spritesheets()?;

        for sprite_sheet_data in sprite_sheet.sprites.into_iter() {
            if let Ok(sprite_name) = SpriteName::from_str(&sprite_sheet_data.name) {
                // Find our Sprite or create a default sprite
                let sprite_metadata = serialized_sprites
                    .iter()
                    .find(|ss| ss.sprite_name == sprite_name)
                    .cloned()
                    .unwrap_or_else(|| SpriteInGameData::create_default(&sprite_sheet_data, sprite_name));

                let data = SpriteData::from_sprite_resource(
                    sprite_sheet_data,
                    sprite_metadata,
                    sprite_name,
                    TextureInformation {
                        page: texture_page_handle,
                        dimensions: Vec2::new(
                            sprite_sheet.texture_width as f32,
                            sprite_sheet.texture_height as f32,
                        ),
                    },
                );
                self.sprites.insert(sprite_name, data);
            } else {
                error!(
                    "A file named {} was in SpriteSheet, but we have no Enum to refer to it!",
                    sprite_sheet_data.name
                );
            }
        }

        if cfg!(debug_assertions) {
            for this_sprite_name in SpriteName::iter() {
                if self.sprites.contains_key(&this_sprite_name) == false {
                    bail!(
                        "We have the SpriteName {} but no sprite data was made for it. This is a hard error.",
                        this_sprite_name
                    )
                }
            }
        }

        Ok(())
    }

    fn initialize_tilesets(&mut self) -> Result<(), Error> {
        let tilesets: Vec<TileSet> = serialization_util::tilesets::load_all_tilesets()?;
        for ts in tilesets {
            self.tilesets.insert(ts.name, ts);
        }

        //  @techdebt only in debug mode. cfg-if?
        let mut reserialize = false;
        let mut serialized_tilesets: Vec<TileSetSerialized> =
            serialization_util::tilesets::load_serialized_tilesets()?;

        for tileset_name in TileSetName::iter() {
            if self.tilesets.contains_key(&tileset_name) == false {
                let new_tileset = TileSet {
                    name: tileset_name,
                    ..Default::default()
                };

                // Insert Tilesets
                self.tilesets.insert(tileset_name, new_tileset.clone());

                // Notify it
                info!("Adding tileset information for {}...", tileset_name);
                serialized_tilesets.push(new_tileset.into());
                reserialize = true;
            }
        }

        if reserialize {
            serialization_util::tilesets::serialize_all_tilesets(&serialized_tilesets)?;
        }

        Ok(())
    }

    pub fn initialize_sounds(&mut self) -> Result<(), Error> {
        for this_sound in SoundResource::iter() {
            let sound_file = this_sound.get_sound_file();
            self.sounds.insert(this_sound, Cursor::new(sound_file));
        }

        Ok(())
    }

    pub fn initialize_fonts(&mut self) -> Result<(), Error> {
        for font in FontName::iter() {
            let font_file = fs::read(&format!("assets/fonts/{}.ttf", font.to_string()))?;
            let glyph_brush = GlyphBrushBuilder::using_font_bytes(font_file).build();

            self.fonts.insert(font, FontData::new(glyph_brush));
        }

        Ok(())
    }

    pub fn load_spritesheets() -> Result<Vec<u8>, Error> {
        // @techdebt this needs to be a config file or something...
        const PACKED_SHEET_DIR: &str = "assets/gen/textures/packed_sheet_0.png";
        match std::fs::read(PACKED_SHEET_DIR) {
            Ok(r) => Ok(r),
            Err(e) => bail!(
                "Couldn't load Spritesheet. {}\nRequested Directory is Packed Sheet Directory: {}",
                e,
                PACKED_SHEET_DIR
            ),
        }
    }

    pub fn prefabs(&self) -> &PrefabMap {
        &self.prefabs
    }

    /// Returns Some when Scene is in Draft Mode -- otherwise,
    /// returns a None.
    pub fn prefabs_mut(&mut self) -> Option<&mut PrefabMap> {
        // compile_error!("We need to have the inspecting mode set here!");
        Some(&mut self.prefabs)
    }

    /// This action adds a prefab to cold storage. Note: it does not serialize the prefab.
    /// You'll have to do that on your own. As always, the file is the definitive prefab list,
    /// and this just exists as a cache.
    pub fn add_prefab(&mut self, prefab: Prefab) {
        self.prefabs.insert(prefab.root_id(), prefab);
    }

    /// This action removes a prefab from the prefab cache. Note: it does not serialize the prefab.
    /// You'll have to do that on your own. As always, the file is the definitive prefab list,
    /// and this just exists as a cache.
    pub fn remove_prefab(&mut self, prefab_id: &uuid::Uuid) -> Option<Prefab> {
        self.prefabs.remove(prefab_id)
    }
}
