use super::{
    graph::Graph,
    relations::insert_with_neighbors,
    siblings_range::SiblingsRange,
    traverse::{Ancestors, Children},
    NodeError,
};
use std::fmt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct NodeId {
    index: usize,
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl NodeId {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn index(self) -> usize {
        self.index
    }

    /// Returns an iterator of references to this node and its ancestors.
    ///
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    pub fn ancestors<T>(self, scene_graph: &Graph<T>) -> Ancestors<'_, T> {
        Ancestors::new(scene_graph, self)
    }

    /// Returns an iterator of references to this node’s children.
    pub fn children<T>(self, scene_graph: &Graph<T>) -> Children<'_, T> {
        Children::new(scene_graph, self)
    }

    /// Detaches a node from its parent. Children are not affected.
    pub fn detach<T>(self, scene_graph: &mut Graph<T>) {
        let range = SiblingsRange::new(self, self).detach_from_siblings(scene_graph);
        range
            .rewrite_parents(scene_graph, None)
            .expect("Should never happen: `None` as parent is always valid");

        // Ensure the node is surely detached.
        debug_assert!(
            scene_graph[self].is_detached(),
            "The node should be successfully detached"
        );
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new child is `self`, or
    /// * the current node or the given new child was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`].
    pub fn append<T>(self, new_child: Self, scene_graph: &mut Graph<T>) {
        self.checked_append(new_child, scene_graph)
            .expect("Preconditions not met: invalid argument");
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::AppendSelf`] error if the given new child is
    ///   `self`.
    /// * Returns [`NodeError::Removed`] error if the given new child or `self`
    ///   is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`].
    pub fn checked_append<T>(self, new_child: Self, scene_graph: &mut Graph<T>) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::AppendSelf);
        }
        if scene_graph[self].is_removed() || scene_graph[new_child].is_removed() {
            return Err(NodeError::Removed);
        }
        new_child.detach(scene_graph);
        insert_with_neighbors(
            scene_graph,
            new_child,
            Some(self),
            scene_graph[self].last_child,
            None,
        )
        .expect("Should never fail: `new_child` is not `self` and they are not removed");

        Ok(())
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new child is `self`, or
    /// * the current node or the given new child was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`].
    pub fn prepend<T>(self, new_child: Self, scene_graph: &mut Graph<T>) {
        self.checked_prepend(new_child, scene_graph)
            .expect("Preconditions not met: invalid argument");
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::PrependSelf`] error if the given new child is
    ///   `self`.
    /// * Returns [`NodeError::Removed`] error if the given new child or `self`
    ///   is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`].
    pub fn checked_prepend<T>(self, new_child: Self, scene_graph: &mut Graph<T>) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::PrependSelf);
        }
        if scene_graph[self].is_removed() || scene_graph[new_child].is_removed() {
            return Err(NodeError::Removed);
        }
        insert_with_neighbors(
            scene_graph,
            new_child,
            Some(self),
            None,
            scene_graph[self].first_child,
        )
        .expect("Should never fail: `new_child` is not `self` and they are not removed");

        Ok(())
    }
}
