use super::{Approach, ComponentList, Follow, Name, Transform, Vec2};
use log_once::error_once;

pub fn update_follows(
    afs: &mut ComponentList<Follow>,
    transforms: &mut ComponentList<Transform>,
    names: &ComponentList<Name>,
    delta_time: f32,
) {
    for follow_c in afs.iter_mut() {
        let id = follow_c.entity_id();
        if let Some(target) = &follow_c.inner_mut().target.target {
            if let Some(target_position) = transforms.get(&target).map(|tc| tc.inner().world_position()) {
                let target_position = target_position + follow_c.inner().offset;

                // Transform Components
                let our_transform_c = transforms.get_mut_or_default(&follow_c.entity_id());
                let transform_pos: &mut Vec2 = &mut our_transform_c.inner_mut().world_position();

                match follow_c.inner().approach {
                    Approach::Instant => {
                        *transform_pos = target_position;
                    }

                    Approach::Linear(speed) => {
                        transform_pos.approached(target_position, Vec2::with_single(speed));
                    }

                    Approach::Asymptotic(weight) => {
                        transform_pos.asymptotic_moved(target_position, weight * delta_time);
                        if (*transform_pos - target_position).magnitude_squared() < delta_time {
                            *transform_pos = target_position;
                        }
                    }
                };

            // Move
            } else {
                error_once!(
                    "{} couldn't find a transform on {} for its Asymptote follow. We need it to have a transform on it!",
                    Name::get_name_quick(names, &id),
                    Name::get_name_quick(names, &target)
                );
            }
        }
    }
}
