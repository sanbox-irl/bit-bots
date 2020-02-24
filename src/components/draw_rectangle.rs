use super::{
    Color, ComponentBounds, DrawOrder, InspectorParameters, Rect, StandardQuad, StandardQuadFactory,
    TextureDescription, Vec2,
};

#[derive(Debug, Clone,SerializableComponent, PartialEq, Serialize, Deserialize, Default, typename::TypeName)]
pub struct DrawRectangle {
    pub rect: Rect,
    pub draw_order: DrawOrder,
    pub tint: Color,
    pub bind_to_bounding_box: bool,
}

use imgui;
impl ComponentBounds for DrawRectangle {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        let mut our_color: [f32; 4] = self.tint.clone().into();
        if imgui::ColorEdit::new(
            &imgui::im_str!("Tint##Draw Rectangle Inspector{}", uid),
            &mut our_color,
        )
        .build(ui)
        {
            self.tint = our_color.into();
        }

        self.rect.rect_inspector(ui, uid);
        self.draw_order.inspect(ui, uid);

        ui.checkbox(
            &imgui::im_str!("Bind to Bounding Box##{}", uid),
            &mut self.bind_to_bounding_box,
        );
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .draw_rectangle
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.draw_rectangle = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.draw_rectangle = None;
    }
}

impl StandardQuadFactory for DrawRectangle {
    fn to_standard_quad(&self, pos: Vec2) -> StandardQuad {
        let translated_box: Rect = self.rect + pos;
        let size = translated_box.size();

        StandardQuad {
            pos: translated_box.min,
            image_size: size,
            draw_order: self.draw_order,
            color: self.tint,
            texture_info: TextureDescription::White,
        }
    }
}
