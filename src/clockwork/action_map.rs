use super::{
    cardinals::{CardinalPrime, FacingHorizontal},
    hardware_interfaces::KeyboardInput,
};
use winit::event::VirtualKeyCode as VK;

#[derive(Debug, Default)]
pub struct ActionMap {
    pub redo: bool,
    pub move_direction: Option<CardinalPrime>,
    pub switch_active_player: Option<FacingHorizontal>,
}

impl ActionMap {
    pub fn update(&mut self, kb: &KeyboardInput) {
        *self = ActionMap::default();

        for key in kb.pressed_keys.iter() {
            self.move_direction = match key {
                VK::Left | VK::A => Some(CardinalPrime::Left),
                VK::Right | VK::D => Some(CardinalPrime::Right),
                VK::Down | VK::S => Some(CardinalPrime::Down),
                VK::Up | VK::W => Some(CardinalPrime::Up),
                _ => None,
            }
        }

        if kb.is_pressed(VK::Q) {
            self.switch_active_player = Some(FacingHorizontal::Left);
        }

        if kb.is_pressed(VK::E) {
            self.switch_active_player = Some(FacingHorizontal::Right);
        }

        self.redo = kb.is_pressed(VK::Z);
    }
}
