use super::{Rect, Vec2};

#[derive(
    Debug, Clone, SerializableComponent, Copy, Default, PartialEq, Serialize, Deserialize, typename::TypeName,
)]
#[serde(default)]
pub struct RelativeBoundingBox {
    pub rect: Rect,
}

impl RelativeBoundingBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            rect: Rect::new(min, max),
        }
    }

    pub fn into_rect(self, native_size: Vec2) -> Rect {
        Rect {
            min: self.rect.min.cwise_product(native_size),
            max: self.rect.max.cwise_product(native_size),
        }
    }
}
