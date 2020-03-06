use super::{
    imgui_component_utils::InspectorParameters,
    scene_graph::{NodeId, SceneGraph},
    ComponentBounds, ComponentList, Entity, Vec2,
};

#[derive(
    Debug,
    SerializableComponent,
    NonSceneGraphComponent,
    Clone,
    Default,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
#[serde(default)]
pub struct Transform {
    local_position: Vec2,
    world_position: Vec2,
    // orientation
    // probably some other garbanzo
    #[serde(skip)]
    dirty: bool,
    #[serde(skip)]
    scene_graph_node_id: Option<NodeId>,
}

impl Transform {
    pub fn new(local_position: Vec2) -> Self {
        Transform {
            local_position,
            world_position: Vec2::ZERO,
            dirty: true,
            scene_graph_node_id: None,
        }
    }

    pub fn world_position(&self) -> Vec2 {
        self.world_position
    }

    pub fn set_local_position(&mut self, new_local_position: Vec2) {
        self.local_position = new_local_position;
    }

    pub fn local_position(&self) -> Vec2 {
        self.local_position
    }

    pub fn edit_local_position(&mut self, f: impl Fn(Vec2) -> Vec2) {
        self.local_position = f(self.local_position);
    }

    pub fn update_world_position(&mut self, parent_position: Vec2) -> Vec2 {
        self.world_position = self.local_position + parent_position;
        self.dirty = false;
        self.world_position
    }

    pub fn local_position_fast(clist: &ComponentList<Transform>, entity_id: &Entity) -> Option<Vec2> {
        clist.get(entity_id).as_ref().map(|&t| t.inner().local_position)
    }

    /// This ADDS to the scene graph -- it doesn't move it around!
    pub fn attach_to_graph(&mut self, entity_id: Entity, scene_graph: &mut SceneGraph) {
        debug_assert_eq!(
            self.scene_graph_node_id, None,
            "Only call this function when NodeID is None!"
        );
        self.scene_graph_node_id = Some(scene_graph.instantiate_node(entity_id));
    }

    /// This ADDS to the scene graph -- it doesn't move it around!
    pub fn attach_to_graph_with_parent(
        &mut self,
        entity_id: Entity,
        parent_id: &NodeId,
        scene_graph: &mut SceneGraph,
    ) {
        debug_assert_eq!(
            self.scene_graph_node_id, None,
            "Only call this function when NodeID is None!"
        );

        let new_node = scene_graph.instantiate_node(entity_id);
        parent_id.append(new_node, scene_graph);
        self.scene_graph_node_id = Some(new_node);
    }

    pub fn scene_graph_node_id(&self) -> Option<NodeId> {
        self.scene_graph_node_id
    }
}

use imgui::*;
impl ComponentBounds for Transform {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if self
            .local_position
            .inspector(ip.ui, &im_str!("Position##{}", ip.uid))
        {
            self.dirty = true;
        }

        self.world_position
            .no_interact_inspector(ip.ui, &im_str!("World Position##{}", ip.uid));
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity.transform.as_ref().map_or(false, |s| {
            s.active == active
                && s.inner.world_position == self.world_position
                && s.inner.local_position == s.inner.local_position
        })
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.transform = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.transform = None;
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
        if self.scene_graph_node_id == other.scene_graph_node_id {
            self.local_position == other.local_position
        } else {
            false
        }
    }
}
