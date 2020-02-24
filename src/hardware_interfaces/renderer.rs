use super::{sprite_resources::SpriteName, Camera, Color, DrawOrder, Vec2, Vec2Int};

macro_rules! manual_drop {
    ($this_val:expr) => {
        ManuallyDrop::into_inner(read(&$this_val))
    };
}

macro_rules! manual_new {
    ($this_val:ident) => {
        ManuallyDrop::new($this_val)
    };
}

mod buffer_bundle;
mod loaded_image;
mod pipeline_bundle;
mod push_constants;
mod renderer_component;
mod renderer_errors;
mod standard_quad;
mod vertex;

pub use buffer_bundle::*;
pub use loaded_image::*;
pub use pipeline_bundle::*;
pub use push_constants::*;
pub use renderer_component::RendererComponent;
pub use renderer_errors::*;
pub use standard_quad::*;
pub use vertex::*;
