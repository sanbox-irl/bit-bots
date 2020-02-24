use super::{
    number_util, sprite_resources::FrameData, Axis, ComponentList, ResourcesDatabase, Sprite, StandardQuad,
    StandardTexture, TextureDescription, Vec2,
};

pub fn update_sprites<'a>(
    sprites: &mut ComponentList<Sprite>,
    resources: &ResourcesDatabase,
    delta_time: f32,
) {
    for this_sprite_component in sprites.iter_mut() {
        let this_sprite = this_sprite_component.inner_mut();

        // Sprite Animation
        if this_sprite.running_data.is_animating && this_sprite.sprite_name.is_some() {
            update_sprite_animation(this_sprite, resources, delta_time);
        }
    }
}

pub fn to_standard_quad(sprite: &Sprite, pos: Vec2, resources: &ResourcesDatabase) -> StandardQuad {
    // Default
    let mut standard_quad = StandardQuad {
        pos,
        draw_order: sprite.running_data.draw_order,
        color: sprite.running_data.tint,
        image_size: Vec2::ZERO,
        texture_info: TextureDescription::White,
    };

    if let Some(sprite_name) = &sprite.sprite_name {
        if let Some(sprite_data) = resources.sprites.get(sprite_name) {
            let current_frame: &FrameData = {
                let current_frame =
                    number_util::wrap_usize(sprite.running_data.current_frame, 0, sprite_data.frames.len());
                &sprite_data.frames[current_frame]
            };

            standard_quad.texture_info = TextureDescription::Standard(StandardTexture {
                norm_image_coordinate: current_frame.normalized_coord,
                norm_image_size: sprite_data.normalized_dimensions,
                texture_page: sprite_data.texture_page.unwrap(),
            });

            standard_quad.pos = pos
                - sprite_data
                    .origin
                    .gfx_adjustment(sprite_data.size)
                    .cwise_product(sprite.running_data.scale);

            standard_quad.image_size = {
                let mut size: Vec2 = Vec2::from(sprite_data.size).cwise_product(sprite.running_data.scale);
                if sprite.running_data.facing_horizontal != sprite_data.facing_horizontal {
                    standard_quad.pos.x += size.x;
                    size.reflected(Axis::X);
                }

                if sprite.running_data.facing_vertical != sprite_data.facing_vertical {
                    standard_quad.pos.y += size.y;
                    size.reflected(Axis::Y);
                }

                size
            };
        } else {
            log_once::error_once!(
                "We're attempting to get {} from the Resources Sprite Database, but it's not there."
            );
        }
    }

    standard_quad
}

fn update_sprite_animation(sprite: &mut Sprite, resources: &ResourcesDatabase, delta_time: f32) {
    if let Some(sprite_data) = resources.sprites.get(sprite.sprite_name.as_ref().unwrap()) {
        let this_frame_data = &sprite_data.frames[sprite.running_data.current_frame];

        // ACTUAL LOOP
        if this_frame_data.duration != 0.0 {
            sprite.running_data.frame_time += delta_time;
            while sprite.running_data.frame_time > this_frame_data.duration {
                sprite.running_data.current_frame += 1;
                if sprite.running_data.current_frame == sprite_data.frames.len() {
                    sprite.running_data.current_frame = 0;
                }
                sprite.running_data.frame_time -= this_frame_data.duration;
            }
        }
    }
}
