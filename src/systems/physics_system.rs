use super::{
    physics_components::BoundingBox, Axis, ComponentList, Entity, PositionalRect, Rect,
    Transform, Vec2,
};

// @techdebt. We need a better solution than this!
pub fn create_positional_rect_lists(
    entity_id: Entity,
    transforms: &ComponentList<Transform>,
    object_bbs: &ComponentList<BoundingBox>,
    // tilemaps: &ComponentList<Tilemap>,
) -> (PositionalRect, Vec<PositionalRect>) {
    let mut out_list = Vec::new();
    let mut our_bb = None;

    for this_comp in object_bbs.iter() {
        let pos = transforms
            .get(&this_comp.entity_id())
            .unwrap()
            .inner()
            .world_position();
        let rect = this_comp.inner().rect;

        let this_positional_rec = PositionalRect::new(pos, rect);

        if this_comp.entity_id() == entity_id {
            our_bb = Some(this_positional_rec);
        } else {
            out_list.push(this_positional_rec);
        }
    }

    // for this_comp in tilemaps.iter() {
    //     for bb in this_comp.inner().collision_bounding_boxes.iter() {
    //         out_list.push(*bb);
    //     }
    // }

    (our_bb.unwrap(), out_list)
}

pub fn move_towards<'a>(
    move_amount: Vec2,
    mut my_bounding_box: PositionalRect,
    bounding_boxes: &Vec<PositionalRect>,
) -> Vec2 {
    // X Movement
    if move_amount.x != 0.0 {
        move_towards_single_axis(move_amount, Axis::X, &mut my_bounding_box, bounding_boxes);
    };

    // Y Movement
    if move_amount.y != 0.0 {
        move_towards_single_axis(move_amount, Axis::Y, &mut my_bounding_box, bounding_boxes);
    }
    my_bounding_box.position()
}

pub fn move_towards_single_axis<'a>(
    move_amount: Vec2,
    axis: Axis,
    my_bounding_box: &mut PositionalRect,
    bounding_boxes: &Vec<PositionalRect>,
) {
    let move_amount_axis = move_amount.get_axis(axis);
    let move_vec = Vec2::with_axis(move_amount_axis, axis);

    if check_collision_at_point(my_bounding_box.rect_at_position() + move_vec, &bounding_boxes) {
        const EXACTNESS: f32 = 10.0;
        let repeat_amount = (move_amount_axis.abs() * EXACTNESS) as usize;
        let move_amount_per_loop = move_amount_axis.signum() / EXACTNESS;

        for _ in 0..repeat_amount {
            let move_vec = Vec2::with_axis(move_amount_per_loop, axis);
            if !check_collision_at_point(my_bounding_box.rect_at_position() + move_vec, &bounding_boxes) {
                my_bounding_box.set_position(my_bounding_box.position() + move_vec);
            } else {
                break;
            }
        }
    } else {
        my_bounding_box.set_position(my_bounding_box.position() + move_vec);
    }
}

pub fn check_collision_at_point<'a>(my_bounding_box: Rect, bounding_boxes: &Vec<PositionalRect>) -> bool {
    for bb_at_pos in bounding_boxes {
        if phy_collisions::rectangle_in_rectangle(my_bounding_box, bb_at_pos.rect_at_position()) {
            return true;
        }
    }
    false
}

// @techdebt big ol techdebt here!
pub fn merge_bounding_boxes(input_boxes: &[Rect], output_vec: &mut Vec<Rect>) {
    for this_rect in input_boxes {
        output_vec.push(this_rect.clone());
    }
}

pub mod phy_collisions {
    use super::{Rect, Vec2};

    #[inline]
    pub fn rectangle_in_rectangle(a: Rect, b: Rect) -> bool {
        !(a.min.x > b.max.x || a.min.y > b.max.y || a.max.x < b.min.x || a.max.y < b.min.y)
    }

    #[inline]
    pub fn point_in_rectangle(rect: &Rect, point: &Vec2) -> bool {
        rect.min.x < point.x && rect.max.x > point.x && rect.min.y < point.y && rect.max.y > point.y
    }

    #[inline]
    pub fn point_in_vec(initial_point: Vec2, point: &Vec2) -> bool {
        point_in_rectangle(&Rect::from_zero_width(initial_point), &point)
    }
}
