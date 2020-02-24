use super::{
    component_utils::RawComponent, ComponentBounds, ComponentList, Entity, GraphNode, InspectorParameters,
    TransformParent, Vec2,
};

#[derive(Debug, SerializableComponent, Clone, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Transform {
    local_position: Vec2,
    world_position: Vec2,
    // orientation
    // probably some other garbanzo
    #[serde(skip)]
    dirty: bool,
    parent: TransformParent,
}

impl Transform {
    pub const TILE: Vec2 = Vec2::new(8.0, 8.0);
    pub const TILE_RIGHT: Vec2 = Vec2::new(8.0, 0.0);
    pub const TILE_UP: Vec2 = Vec2::new(0.0, 8.0);

    pub fn new(local_position: Vec2) -> Self {
        Transform {
            local_position,
            world_position: Vec2::ZERO,
            dirty: true,
            parent: TransformParent::blank(),
        }
    }

    pub(super) fn set_new_parent(&mut self, my_id: Entity, new_parent_node: RawComponent) {
        // Dirty the Transform, cause it needs to be moved again!
        self.dirty = true;

        self.remove_self_from_parent(my_id);

        // Now Set the new Parent:
        self.parent.target = new_parent_node;
    }

    fn remove_self_from_parent(&mut self, my_id: Entity) {
        // Remove the old Parent!
        if let Some(parent) = self.parent_mut() {
            let children = parent.children.as_mut().unwrap();
            let pos = children
                .iter()
                .position(|ser| ser.target.map(|target| target == my_id).unwrap_or_default());

            if let Some(pos) = pos {
                children.remove(pos);
            } else {
                error!("Entity {} had a parent, but it was not their parent.", my_id);
            }
        }
    }

    pub fn parent_exists(&self) -> bool {
        self.parent.target.is_real()
    }

    pub fn parent_mut(&mut self) -> Option<&mut GraphNode> {
        self.parent.parent_mut()
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

    pub fn local_position_fast(clist: &ComponentList<Transform>, entity_id: &Entity) -> Option<Vec2> {
        clist.get(entity_id).as_ref().map(|&t| t.inner().local_position)
    }

    pub fn update_world_position(&mut self, parent_position: Vec2) -> Vec2 {
        self.world_position = self.local_position + parent_position;
        self.dirty = false;
        self.world_position()
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
        serialized_entity
            .transform
            .as_ref()
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        let clone = {
            let mut clone = self.clone();
            if self.parent.is_root() {
                clone.parent = TransformParent::default();
                clone.dirty = false;
            }
            clone
        };

        // Copy it all over:
        se.transform = Some(super::SerializedComponent { inner: clone, active });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.transform = None;
    }
    
    fn post_deserialization(
        &mut self,
        entity: super::Entity,
        serialization_markers: &super::ComponentList<super::SerializationMarker>,
    ) {
        super::scene_graph::add_to_scene_graph((self, entity), serialization_markers);
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Transform) -> bool {
        if self.parent == other.parent {
            self.local_position == other.local_position
        } else {
            false
        }
    }
}