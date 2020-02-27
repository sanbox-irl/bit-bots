use super::{Node, NodeId, SceneGraph};

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

#[derive(Clone)]
/// An iterator of references to the children of a given node.
pub struct NodeChildren<'a> {
    scene_graph: &'a SceneGraph,
    node_id: Option<NodeId>,
}

impl<'a> NodeChildren<'a> {
    pub(super) fn new(scene_graph: &'a SceneGraph, current: &Node) -> Self {
        Self {
            scene_graph,
            node_id: current.first_child,
        }
    }
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<&'a Node> {
        let node: &Node = &self.scene_graph[self.node_id.take()?];

        self.node_id = node.next_sibling;
        Some(node)
    }
}
