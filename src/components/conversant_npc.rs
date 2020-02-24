use super::{
    component_utils::SerializableEntityReference, Color, ComponentBounds, Entity, InspectorParameters,
    SerializablePrefabReference,
};

#[derive(
    Debug, Clone, SerializableComponent, Default, PartialEq, Serialize, Deserialize, typename::TypeName,
)]
#[serde(default)]
pub struct ConversantNPC {
    pub conversation_partner: SerializableEntityReference,
    pub initial_ui_prefab: SerializablePrefabReference,
    pub text_ui_prefab: SerializablePrefabReference,
    pub distance: f32,
    pub color_on_close: Color,
    pub color_on_far: Color,
    pub converse_with_input: bool,
    pub converse_text: String,

    #[serde(skip)]
    pub runtime_ui: Option<Entity>,
}

impl ComponentBounds for ConversantNPC {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        // pub ui_component_entity: SerializableEntityReference
        self.conversation_partner.inspect("Conversation Partner", &ip);

        // pub initial_ui_prefab: SerializablePrefabReference,
        self.initial_ui_prefab.inspect("Bang UI", &ip);
        // pub text_ui_prefab: SerializablePrefabReference,
        self.text_ui_prefab.inspect("Text UI", &ip);

        // pub distance: f32
        ip.ui
            .drag_float(&imgui::im_str!("Distance##{}", ip.uid), &mut self.distance)
            .build();

        // pub color_on_close: Color
        self.color_on_close.inspect(&ip.ui, "Color Close", ip.uid);

        // pub color_on_far: Color
        self.color_on_far.inspect(&ip.ui, "Color Far", ip.uid);

        // pub converse_with_input: bool
        ip.ui.checkbox(
            &imgui::im_str!("Converse With Input##{}", ip.uid),
            &mut self.converse_with_input,
        );
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .conversant_npc
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        serialization_marker: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.conversant_npc = Some({
            let mut clone: super::ConversantNPC = self.clone();
            clone
                .conversation_partner
                .entity_id_to_serialized_refs(&serialization_marker);

            super::SerializedComponent { inner: clone, active }
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.conversant_npc = None;
    }

    fn post_deserialization(
        &mut self,
        _: super::Entity,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        self.conversation_partner
            .serialized_refs_to_entity_id(serialization_markers);
    }
}
