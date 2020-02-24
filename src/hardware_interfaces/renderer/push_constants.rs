use super::{Camera, Color, StandardQuad, StandardTexture, Vec2};

// Don't mess with this without updating the
// equivalent shader!
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct StandardPushConstants {
    pub camera_position: Vec2,
    pub camera_native_resolution: Vec2,
    pub entity_position: Vec2,
    pub image_size: Vec2,
    pub norm_image_coord: Vec2,
    pub norm_image_size: Vec2,
    pub color: Color,
}

impl StandardPushConstants {
    pub fn with_camera_data(camera_position: Vec2, camera_data: &Camera) -> Self {
        Self {
            camera_position,
            camera_native_resolution: camera_data.ingame_camera_size(),
            ..Default::default()
        }
    }

    pub fn update(&mut self, standard_quad: &StandardQuad, texture_info: &StandardTexture) {
        self.color = standard_quad.color;
        self.entity_position = standard_quad.pos;
        self.image_size = standard_quad.image_size;
        self.norm_image_coord = texture_info.norm_image_coordinate;
        self.norm_image_size = texture_info.norm_image_size;
    }

    pub fn to_bits(self) -> [u32; std::mem::size_of::<Self>() / std::mem::size_of::<u32>()] {
        unsafe { std::mem::transmute(self) }
    }
}

// Don't mess with this without updating the
// equivalent shader!
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct ImguiPushConstants {
    pub scale: Vec2,
    pub translate: Vec2,
}

impl ImguiPushConstants {
    pub fn to_bits(self) -> [u32; std::mem::size_of::<Self>() / std::mem::size_of::<u32>()] {
        unsafe { std::mem::transmute(self) }
    }
}
