use super::{Camera, CameraMode, ComponentList, Entity, Follow, Input, MouseButton, Transform};
use winit::event::VirtualKeyCode as VK;

pub fn update_camera(
    camera: &mut Camera,
    camera_entity: &Entity,
    transforms: &mut ComponentList<Transform>,
    follows: &mut ComponentList<Follow>,
    input: &Input,
) {
    match camera.current_mode {
        CameraMode::Standard => {
            if let Some(follow_c) = follows.get_mut(camera_entity) {
                follow_c.is_active = true;
            }

            // Pixel correction here @techdebt
            transforms
                .get_mut_or_default(camera_entity)
                .inner_mut()
                .edit_local_position(|pos| (pos * 32.0).floor() / 32.0);
        }
        CameraMode::Debug => {
            if let Some(follow_c) = follows.get_mut(camera_entity) {
                follow_c.is_active = false;
            }

            // Scrolling
            if input.mouse_input.mouse_vertical_scroll_delta != 0.0 {
                // this is a random heuristic so it's less annoying
                camera.zoom_level -= input.mouse_input.mouse_vertical_scroll_delta / 100.0;
                camera.zoom_level = f32::max(camera.zoom_level, 0.0);
            }

            // Panning
            if input.mouse_input.is_held(MouseButton::Middle)
                || input.kb_input.is_held(VK::LAlt)
                || input.kb_input.is_held(VK::RAlt)
            {
                transforms
                    .get_mut_or_default(camera_entity)
                    .inner_mut()
                    .edit_local_position(|associated_position| {
                        let old_pos = camera.display_to_world_position(
                            input.mouse_input.mouse_position_last_frame,
                            associated_position,
                        );

                        let new_pos = camera
                            .display_to_world_position(input.mouse_input.mouse_position, associated_position);

                        associated_position + (old_pos - new_pos)
                    });
            }
        }
    }
}
