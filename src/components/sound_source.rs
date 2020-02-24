use super::*;

#[derive(
    Debug,
    SerializableComponent,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    typename::TypeName,
    Hash,
)]
#[serde(default)]
pub struct SoundSource {
    pub sound_to_play: Option<SoundResource>,
    pub muted: bool,
}

use imgui::*;
impl ComponentBounds for SoundSource {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        if let Some(sound_to_play_maybe) =
            imgui_system::typed_enum_selection_option(ui, &self.sound_to_play, uid)
        {
            self.sound_to_play = sound_to_play_maybe;
        }

        // MUTED
        ui.checkbox(&im_str!("Muted##{}", uid), &mut self.muted);
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .sound_source
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.sound_source = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.sound_source = None;
    }
}
