use super::{
    component_utils::{TextHorizontalAlign, TextVerticalAlign},
    fonts::FontName,
    imgui_system, Color, ComponentBounds, DrawOrder, InspectorParameters, StandardQuad, Vec2,
};

#[derive(
    Debug, SerializableComponent, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName,
)]
#[serde(default)]
pub struct TextSource {
    pub font: FontName,
    pub text: String,
    pub scale: Vec2,
    pub screen_scale: f32,
    pub color: Color,
    pub horizontal_align: TextHorizontalAlign,
    pub vertical_align: TextVerticalAlign,
    pub draw_order: DrawOrder,

    #[serde(skip)]
    pub cached_quads: Vec<StandardQuad>,
}

impl ComponentBounds for TextSource {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        let mut current_text = imgui::ImString::new(&self.text);
        if ip
            .ui
            .input_text(&imgui::im_str!("Text##{}", ip.uid), &mut current_text)
            .build()
        {
            // @techdebt figure this out!
            self.text = (&current_text).to_str().to_string();
        }

        if let Some(new_font) = imgui_system::typed_enum_selection(&ip.ui, &self.font, ip.uid) {
            self.font = new_font;
        }

        //pub scale: Vec2
        self.scale.inspector(&ip.ui, &imgui::im_str!("Scale##{}", ip.uid));

        //  pub screen_mod: f32
        ip.ui
            .drag_float(
                &imgui::im_str!("Screen Scale##{}", ip.uid),
                &mut self.screen_scale,
            )
            .build();

        if self.screen_scale < 1.0 {
            self.screen_scale = 1.0;
        }

        self.color.inspect(&ip.ui, "Color", ip.uid);

        if let Some(new_horizontal) =
            imgui_system::typed_enum_selection(&ip.ui, &self.horizontal_align, ip.uid)
        {
            self.horizontal_align = new_horizontal;
        }

        if let Some(new_vertical) = imgui_system::typed_enum_selection(&ip.ui, &self.vertical_align, ip.uid) {
            self.vertical_align = new_vertical;
        }

        self.draw_order.inspect(&ip.ui, ip.uid);
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .text_source
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.text_source = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.text_source = None;
    }
}

impl TextSource {
    pub fn prepare_standard_quad(&self, pos: Vec2, quad: &StandardQuad) -> StandardQuad {
        let mut ret = quad.clone();

        ret.pos /= self.screen_scale;
        ret.pos += pos;
        ret.image_size /= self.screen_scale;
        ret.color = self.color;
        ret.draw_order = self.draw_order;

        ret
    }
}
