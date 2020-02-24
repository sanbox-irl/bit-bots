use super::{input::Input, renderer::RendererComponent, window::Window};
use anyhow::Error;

pub struct HardwareInterface {
    pub input: Input,
    pub window: Window,
    pub renderer: RendererComponent,
    // pub sound_player: SoundPlayer,
}

impl HardwareInterface {
    pub fn new(config: &super::game_config::Config) -> Result<Self, Error> {
        let window = Window::new(config.window_size)?;
        let renderer = RendererComponent::typed_new(&window.window)?;
        // let sound_player = SoundPlayer::new();

        info!("✔ Initialized Hardware Resources");

        Ok(Self {
            input: Input::new(),
            window,
            renderer,
            // sound_player,
        })
    }
}
