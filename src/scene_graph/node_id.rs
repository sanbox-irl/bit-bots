use super::{relations::insert_with_neighbors, Entity, Node, NodeError, SceneGraph, SiblingsRange};
use std::fmt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_1_1_1 = arena.new_node("1_1_1_1");
    /// # n1_1_1.append(n1_1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |       `-- 1_1_1_1
    /// //     _-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_1_1.ancestors(&arena);
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    // pub fn ancestors<T>(self, scene_graph: &mut SceneGraph) -> Ancestors<'_, T> {
    //     Ancestors::new(scene_graph, self)
    // }

    /// Returns an iterator of references to this node’s children.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.children(&arena);
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    // pub fn children(self, scene_graph: &SceneGraph) -> Children<'_, Entity> {
    //     Children::new(scene_graph, self)
    // }

    /// Detaches a node from its parent. Children are not affected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- (implicit)
    /// //     `-- 1
    /// //         |-- 1_1
    /// //         |   `-- 1_1_1
    /// //         |-- 1_2 *
    /// //         `-- 1_3
    ///
    /// n1_2.detach(&mut arena);
    /// // arena
    /// // |-- (implicit)
    /// // |   `-- 1
    /// // |       |-- 1_1
    /// // |       |   `-- 1_1_1
    /// // |       `-- 1_3
    /// // `-- (implicit)
    /// //     `-- 1_2
    ///
    /// assert!(arena[n1_2].parent().is_none());
    /// assert!(arena[n1_2].previous_sibling().is_none());
    /// assert!(arena[n1_2].next_sibling().is_none());
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn detach(self, scene_graph: &mut SceneGraph) {
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
    pub fn append(self, new_child: NodeId, scene_graph: &mut SceneGraph) {
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
    pub fn checked_append(self, new_child: NodeId, scene_graph: &mut SceneGraph) -> Result<(), NodeError> {
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
    pub fn prepend(self, new_child: NodeId, scene_graph: &mut SceneGraph) {
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
    pub fn checked_prepend(self, new_child: NodeId, scene_graph: &mut SceneGraph) -> Result<(), NodeError> {
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
