use super::{NodeId, SceneGraph};

#[derive(Clone)]
/// An iterator of references to the ancestors a given node.
pub struct Ancestors<'a> {
    scene_graph: &'a SceneGraph,
    node: Option<NodeId>,
}

impl<'a> Ancestors<'a> {
    pub(crate) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
        Self {
            scene_graph,
            node: Some(current),
        }
    }
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].parent;
        Some(node)
    }
}

#[derive(Clone)]
/// An iterator of references to the children of a given node.
pub struct Children<'a> {
    scene_graph: &'a SceneGraph,
    node: Option<NodeId>,
}

impl<'a> Children<'a> {
    pub(super) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
        Self {
            scene_graph,
            node: scene_graph[current].first_child,
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].next_sibling;
        Some(node)
    }
}

/// An iterator of references to the siblings after a given node.
/// We have this in the codebase essentially only for pretty printing
/// the tree.
#[derive(Clone)]
pub struct Siblings<'a> {
    scene_graph: &'a SceneGraph,
    node: Option<NodeId>,
}
// impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling);

impl<'a> Siblings<'a> {
    pub(super) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
        Self {
            scene_graph,
            node: Some(current),
        }
    }
}

impl<'a> Iterator for Siblings<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].next_sibling;
        Some(node)
    }
}