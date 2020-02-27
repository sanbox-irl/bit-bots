use super::{
    component_utils::RawComponent, imgui_system, scene_graph::NodeId, ComponentBounds, ComponentData,
    ComponentList, Entity, InspectorParameters, SerializableEntityReference, SerializationMarker, Transform,
};

#[derive(
    Debug, Clone, SerializableComponent, PartialEq, Default, Serialize, Deserialize, typename::TypeName,
)]
#[serde(default)]
pub struct GraphNode {
    pub scene_graph_node_id: Option<NodeId>,
}

impl GraphNode {
    // #[allow(dead_code)]
    // pub fn specific_entity_inspector(
    //     &mut self,
    //     entity_id: Entity,
    //     ip: InspectorParameters<'_, '_>,
    //     serializations: &ComponentList<SerializationMarker>,
    //     transforms: &mut ComponentList<Transform>,
    // ) {
    //     if let Some(our_children) = &self.children {
    //         for this_child in our_children {
    //             if let Some(this_child_target) = this_child.target {
    //                 ip.ui
    //                     .bullet_text(&imgui::im_str!("{}##{}", this_child_target, ip.uid));
    //             } else {
    //                 ip.ui.bullet_text(imgui::im_str!("Blank Child!"));
    //             }
    //         }
    //     } else {
    //         ip.ui.text("None");
    //     }

    //     if let Some(new_child) =
    //         imgui_system::select_entity("Add Child", ip.uid, ip.ui, ip.entities, ip.entity_names)
    //     {
    //         self.add_child(Some(entity_id), new_child, transforms, serializations);
    //     }
    // }
}

impl ComponentBounds for GraphNode {
    fn entity_inspector(&mut self, _ip: InspectorParameters<'_, '_>) {
        unimplemented!();
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .graph_node
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        // se.graph_node = Some({
        //     let mut clone: super::GraphNode = self.clone();
        //     if let Some(children) = clone.children.as_mut() {
        //         for child in children.iter_mut() {
        //             child.entity_id_to_serialized_refs(&serialization_markers);
        //         }
        //     }

        //     super::SerializedComponent { inner: clone, active }
        // });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.graph_node = None;
    }

    fn post_deserialization(
        &mut self,
        _: super::Entity,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        if let Some(children) = &mut self.scene_graph_node_id {
            // for child in children.iter_mut() {
            //     child.serialized_refs_to_entity_id(&serialization_markers);
            // }
        }
    }
}
