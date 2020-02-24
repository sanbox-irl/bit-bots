use super::shared::*;
use failure::{bail, Error};
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sheep::{Format, InputSprite, MaxrectsPacker, SpriteAnchor};
use std::fs;

const JSON_GLOB: &str = "assets/textures/**/";
const PNG_FNAME: &str = "assets/gen/textures/packed_sheet_";

pub fn pack_sprites(input_sprites: Vec<InputSprite>, file_names: Vec<String>) -> Result<(), Error> {
    let results = sheep::pack::<MaxrectsPacker>(input_sprites, 4, sheep::MaxrectsOptions::default());

    for (i, sprite_sheet_png) in results.into_iter().enumerate() {
        let mut sprite_sheet_meta: SpriteSheet =
            sheep::encode::<SanboxFormat>(&sprite_sheet_png, file_names.clone());

        // Check if we have an aseprite JSON to merge in...
        for this_file in &mut sprite_sheet_meta.sprites {
            // @techdebt this is weird to use a glob for one file. Hell, maybe it's not.
            let collection = glob(&format!("{}{}.json", JSON_GLOB, this_file.name)).unwrap();

            for this_item in collection.into_iter() {
                let str = fs::read_to_string(this_item.unwrap()).unwrap();
                let aseprite = json_parse(serde_json::from_str(&str).unwrap()).unwrap();
                update_sprite_resource(this_file, aseprite);
            }
        }

        let outbuf = image::RgbaImage::from_vec(
            sprite_sheet_png.dimensions.0,
            sprite_sheet_png.dimensions.1,
            sprite_sheet_png.bytes,
        )
        .expect("Failed to construct image from sprite sheet bytes");

        outbuf.save(format!("{}{}.png", PNG_FNAME, i))?;

        let meta_str = serde_yaml::to_string(&sprite_sheet_meta)?;
        let fpath = format!("{}{}.yaml", PNG_FNAME, i);
        fs::write(&fpath, meta_str)?;
    }

    Ok(())
}

fn json_parse(json: Value) -> Result<AsepriteJSON, Error> {
    if let Value::Object(json_map) = json {
        if let Value::Object(frame_map) = &json_map["frames"] {
            let mut frames = Vec::new();
            for this_key_combo in frame_map {
                let this_frame: Frame = serde_json::from_value(this_key_combo.1.clone())?;
                frames.push(this_frame);
            }

            Ok(AsepriteJSON { frames })
        } else {
            bail!("No frames!");
        }
    } else {
        bail!("Not a JSON map!");
    }
}

fn update_sprite_resource(sprite_resource: &mut SpriteResource, aseprite_json: AsepriteJSON) {
    if aseprite_json.frames.len() == 1 {
        return;
    }

    let update_x = sprite_resource.frames[0].height == sprite_resource.height;

    sprite_resource.frames.clear();

    let mut running_x = sprite_resource.x;
    let mut running_y = sprite_resource.y;

    for this_frame in aseprite_json.frames {
        let frame_dimensions = this_frame.frame;

        sprite_resource.frames.push(FrameResource {
            x: running_x,
            y: running_y,
            width: frame_dimensions.w as u32,
            height: frame_dimensions.h as u32,
        });

        if update_x {
            running_x += frame_dimensions.w as u32;
        } else {
            running_y += frame_dimensions.h as u32;
        }
    }
}

#[derive(Debug)]
struct AsepriteJSON {
    pub frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Frame {
    pub frame: FrameDimensions,
    pub rotated: bool,
    pub trimmed: bool,
    pub sprite_source_size: FrameDimensions,
    pub source_size: Vec2IntWH,
    pub duration: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Meta {
    app: String,
    version: String,
    image: String,
    size: Vec2IntWH,
    scale: String,
    frame_tags: Vec<FrameTags>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
struct Vec2Int {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
struct Vec2IntWH {
    pub w: u64,
    pub h: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct FrameTags {
    pub name: String,
    pub from: u64,
    pub to: u64,
    pub direction: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct FrameDimensions {
    pub x: u64,
    pub y: u64,
    pub w: u64,
    pub h: u64,
}

#[allow(dead_code)]
struct SanboxFormat;
impl Format for SanboxFormat {
    type Data = SpriteSheet;
    type Options = Vec<String>;

    fn encode(dimensions: (u32, u32), sprites: &[SpriteAnchor], options: Self::Options) -> Self::Data {
        let sprite_positions = sprites
            .iter()
            .map(|anchor| (anchor, options[anchor.id].clone()))
            .map(Into::into)
            .collect::<Vec<SpriteResource>>();

        SpriteSheet {
            texture_width: dimensions.0,
            texture_height: dimensions.1,
            sprites: sprite_positions,
        }
    }
}

impl From<(&SpriteAnchor, String)> for SpriteResource {
    fn from(anchor: (&SpriteAnchor, String)) -> SpriteResource {
        SpriteResource {
            name: anchor.1,
            x: anchor.0.position.0,
            y: anchor.0.position.1,
            width: anchor.0.dimensions.0,
            height: anchor.0.dimensions.1,
            frames: vec![FrameResource {
                x: anchor.0.position.0,
                y: anchor.0.position.1,
                width: anchor.0.dimensions.0,
                height: anchor.0.dimensions.1,
            }],
        }
    }
}
