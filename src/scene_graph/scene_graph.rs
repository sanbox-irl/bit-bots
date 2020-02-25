use super::{Entity, Node, NodeId};
use std::ops::{Index, IndexMut};

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct SceneGraph {
    nodes: Vec<Node>,
}

impl SceneGraph {
    /// Creates a new empty `SceneGraph`.
    pub fn new() -> SceneGraph {
        Self { nodes: Vec::new() }
    }

    /// Create a new node on the scene graph. The new Node ID is not
    /// connected to anything and is therefore a "root" node.
    pub fn create_node(&mut self, entity: Entity) -> NodeId {
        let index = self.nodes.len();
        self.nodes.push(Node::new(entity));

        NodeId::new(index)
    }

    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index())
    }

    /// Returns a naive iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }
}

impl Index<NodeId> for SceneGraph {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Node {
        &self.nodes[index.index()]
    }
}

impl IndexMut<NodeId> for SceneGraph {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.index()]
    }
}
