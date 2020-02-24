use super::Vec2;
use winit::event::{ElementState, VirtualKeyCode};

#[derive(Debug)]
pub struct Input {
    pub end_requested: bool,
    pub new_frame_size: Option<Vec2>,
    pub mouse_input: MouseInput,
    pub kb_input: KeyboardInput,
}

impl Input {
    pub fn new() -> Self {
        Self {
            end_requested: false,
            new_frame_size: None,
            mouse_input: MouseInput::default(),
            kb_input: KeyboardInput::default(),
        }
    }

    pub fn clear_input(&mut self) {
        self.end_requested = false;
        self.new_frame_size = None;
        self.mouse_input.clear();
        self.kb_input.clear();
    }

    pub fn record_input(
        &mut self,
        element_state: ElementState,
        code: VirtualKeyCode,
        last_frame_pressed: &[VirtualKeyCode],
    ) {
        match element_state {
            ElementState::Pressed => {
                if let None = last_frame_pressed.iter().position(|&pos| pos == code) {
                    if let None = self.kb_input.held_keys.iter().position(|&pos| pos == code) {
                        self.kb_input.pressed_keys.push(code);
                        self.kb_input.held_keys.push(code);
                    }
                }
            }

            ElementState::Released => {
                if let Some(vk_pos) = self.kb_input.held_keys.iter().position(|&item| item == code) {
                    self.kb_input.held_keys.remove(vk_pos);
                    self.kb_input.released_keys.push(code);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct MouseInput {
    pub mouse_position_last_frame: Vec2,
    pub mouse_position: Vec2,
    pub mouse_vertical_scroll_delta: f32,
    pub mouse_pressed: [bool; 5],
    pub mouse_held: [bool; 5],
    pub mouse_released: [bool; 5],
    pub mouse_input_taken: bool,
}

impl MouseInput {
    pub fn clear(&mut self) {
        for elem in self.mouse_pressed.iter_mut() {
            *elem = false;
        }
        for elem in self.mouse_released.iter_mut() {
            *elem = false;
        }
        self.mouse_position_last_frame = self.mouse_position;
        self.mouse_vertical_scroll_delta = 0.0;
        self.mouse_input_taken = false;
    }

    pub fn clear_held(&mut self) {
        for elem in self.mouse_held.iter_mut() {
            *elem = false;
        }
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_pressed[index] && self.mouse_input_taken == false
    }

    #[allow(dead_code)]
    pub fn is_held(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_held[index] && self.mouse_input_taken == false
    }

    #[allow(dead_code)]
    pub fn is_released(&self, mouse_button: MouseButton) -> bool {
        let index: usize = mouse_button.into();
        self.mouse_released[index] || self.mouse_input_taken
    }

    pub fn mouse_delta_position(&self) -> Vec2 {
        self.mouse_position - self.mouse_position_last_frame
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Generic(usize),
}

impl From<MouseButton> for usize {
    fn from(w: MouseButton) -> usize {
        match w {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Generic(index) => index,
        }
    }
}

#[derive(Debug, Default)]
pub struct KeyboardInput {
    pub pressed_keys: Vec<VirtualKeyCode>,
    pub held_keys: Vec<VirtualKeyCode>,
    pub released_keys: Vec<VirtualKeyCode>,
}

impl KeyboardInput {
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self, target_keycode: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&target_keycode)
    }

    #[allow(dead_code)]
    pub fn is_held(&self, target_keycode: VirtualKeyCode) -> bool {
        self.held_keys.contains(&target_keycode)
    }

    #[allow(dead_code)]
    pub fn is_released(&self, target_keycode: VirtualKeyCode) -> bool {
        self.released_keys.contains(&target_keycode)
    }
}
