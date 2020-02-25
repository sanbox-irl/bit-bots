use super::{Entity, NodeId};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Node {
    pub(super) parent: Option<NodeId>,
    pub(super) first_child: Option<NodeId>,
    pub(super) last_child: Option<NodeId>,
    pub(super) previous_sibling: Option<NodeId>,
    pub(super) next_sibling: Option<NodeId>,
    pub(super) removed: bool,
    pub(super) entity: Entity,
}

impl Node {
    /// Creates a new `Node` with the default state and the given data.
    pub(super) fn new(entity: Entity) -> Self {
        Self {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            removed: false,
            entity,
        }
    }

    /// Returns a reference to the node data.
    pub fn inner(&self) -> &Entity {
        &self.entity
    }

    /// Returns a mutable reference to the entity ID.
    pub fn inner_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }

    /// Returns the ID of the parent node, unless this node is the root of the
    /// tree.
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Returns the ID of the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    /// Returns the ID of the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    /// Checks if the node is marked as removed.
    pub fn is_removed(&self) -> bool {
        self.removed
    }

    /// Checks if the node is detached. `is_detached()` != `parent().is_some()`
    pub(super) fn is_detached(&self) -> bool {
        self.parent.is_none()
    }
}
