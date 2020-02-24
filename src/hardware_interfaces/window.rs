use super::Vec2;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window as WinitWindow, WindowBuilder},
};

const WINDOW_NAME: &'static str = "Bit Bots";

pub struct Window {
    pub name: &'static str,
    pub events_loop: EventLoop<()>,
    pub window: WinitWindow,
}

impl Window {
    pub fn new(size: Vec2) -> Result<Self, anyhow::Error> {
        let events_loop = EventLoop::new();

        let output = WindowBuilder::new()
            .with_title(WINDOW_NAME)
            .with_inner_size(PhysicalSize {
                width: size.x as f64,
                height: size.y as f64,
            })
            .with_resizable(false)
            .build(&events_loop);

        Ok(output.map(|window| Self {
            events_loop,
            window,
            name: WINDOW_NAME,
        })?)
    }

    pub fn get_window_size(&self) -> Vec2 {
        let window_client_area = self.window.inner_size();
        Vec2::new(window_client_area.width as f32, window_client_area.height as f32)
    }
}
