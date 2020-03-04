use super::{graph::Graph, node_id::NodeId, traverse::NodeChildren};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct GraphNode<T> {
    pub(super) parent: Option<NodeId>,
    pub(super) first_child: Option<NodeId>,
    pub(super) last_child: Option<NodeId>,
    pub(super) previous_sibling: Option<NodeId>,
    pub(super) next_sibling: Option<NodeId>,
    pub(super) removed: bool,
    pub(super) data: T,
}

impl<T> GraphNode<T> {
    /// Creates a new `Node` with the default state and the given data.
    pub(super) fn new(data: T) -> Self {
        Self {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            removed: false,
            data,
        }
    }

    /// Returns a reference to the node data.
    pub fn inner(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the entity ID.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.data
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

    /// Returns an iterator of references to this node’s children.
    pub fn children<'a>(&self, scene_graph: &'a Graph<T>) -> NodeChildren<'a, T> {
        NodeChildren::new(scene_graph, self)
    }

    /// Checks if the node is detached. `is_detached()` != `parent().is_some()`
    pub fn is_detached(&self) -> bool {
        self.parent.is_none()
    }
}

impl<T: std::fmt::Display> fmt::Display for GraphNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
