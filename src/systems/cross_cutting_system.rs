use super::{
    physics_components::BoundingBox, ComponentList, DrawRectangle, Ecs, Rect, ResourcesDatabase, Sprite,
};

pub fn cross_cutting_system(ecs: &mut Ecs, resources: &ResourcesDatabase) {
    bounding_box_and_sprite(
        &mut ecs.component_database.bounding_boxes,
        &ecs.component_database.sprites,
        resources,
    );

    draw_rectangle_and_bounding_box(
        &mut ecs.component_database.draw_rectangles,
        &ecs.component_database.bounding_boxes,
    );
}

fn bounding_box_and_sprite(
    bbs: &mut ComponentList<BoundingBox>,
    sprites: &ComponentList<Sprite>,
    resources: &ResourcesDatabase,
) {
    for this_bb in bbs.iter_mut() {
        let this_entity_id = this_bb.entity_id();
        let this_bounding_box: &mut BoundingBox = this_bb.inner_mut();

        if this_bounding_box.bind_to_sprite {
            if let Some(this_sprite) = sprites.get(&this_entity_id) {
                if let Some(sprite_name) = &this_sprite.inner().sprite_name {
                    let sprite_data = resources.sprites.get(sprite_name).unwrap();
                    let rel_location = sprite_data.origin.sprite_location_relative(sprite_data.size);

                    this_bounding_box.rect = Rect::point_width(rel_location, sprite_data.size.into());
                };
            } else {
                this_bounding_box.bind_to_sprite = false;
            }
        }
    }
}

fn draw_rectangle_and_bounding_box(
    draw_rects: &mut ComponentList<DrawRectangle>,
    bbs: &ComponentList<BoundingBox>,
) {
    for this_draw_rect in draw_rects.iter_mut() {
        let this_entity_id = this_draw_rect.entity_id();
        let this_draw_rect_inner: &mut DrawRectangle = this_draw_rect.inner_mut();

        if this_draw_rect_inner.bind_to_bounding_box {
            if let Some(this_bb) = bbs.get(&this_entity_id) {
                this_draw_rect_inner.rect = this_bb.inner().rect;
            } else {
                this_draw_rect_inner.bind_to_bounding_box = false;
            }
        }
    }
}
