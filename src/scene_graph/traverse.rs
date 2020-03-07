use super::{graph::Graph, graph_id::GraphId, node::GraphNode};

#[derive(Clone)]
/// An iterator of references to the ancestors a given node.
pub struct Ancestors<'a, T> {
    scene_graph: &'a Graph<T>,
    node: Option<GraphId<T>>,
}

impl<'a, T> Ancestors<'a, T> {
    pub(crate) fn new(scene_graph: &'a Graph<T>, current: GraphId<T>) -> Self {
        Self {
            scene_graph,
            node: Some(current),
        }
    }
}

impl<'a, T> Iterator for Ancestors<'a, T> {
    type Item = GraphId<T>;

    fn next(&mut self) -> Option<GraphId<T>> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].parent;
        Some(node)
    }
}

#[derive(Clone)]
/// An iterator of references to the children of a given node.
pub struct Children<'a, T> {
    scene_graph: &'a Graph<T>,
    node: Option<GraphId<T>>,
}

impl<'a, T> Children<'a, T> {
    pub(super) fn new(scene_graph: &'a Graph<T>, current: GraphId<T>) -> Self {
        Self {
            scene_graph,
            node: scene_graph[current].first_child,
        }
    }
}

impl<'a, T> Iterator for Children<'a, T> {
    type Item = GraphId<T>;

    fn next(&mut self) -> Option<GraphId<T>> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].next_sibling;
        Some(node)
    }
}

#[derive(Clone)]
/// An iterator of references to the children of a given node.
pub struct NodeChildren<'a, T> {
    scene_graph: &'a Graph<T>,
    node_id: Option<GraphId<T>>,
}

impl<'a, T> NodeChildren<'a, T> {
    pub(super) fn new(scene_graph: &'a Graph<T>, current: &GraphNode<T>) -> Self {
        Self {
            scene_graph,
            node_id: current.first_child,
        }
    }
}

impl<'a, T> Iterator for NodeChildren<'a, T> {
    type Item = &'a GraphNode<T>;

    fn next(&mut self) -> Option<&'a GraphNode<T>> {
        let node: &GraphNode<T> = &self.scene_graph[self.node_id.take()?];

        self.node_id = node.next_sibling;
        Some(node)
    }
}
