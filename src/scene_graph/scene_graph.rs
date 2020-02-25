use super::{traverse::Siblings, Entity, Node, NodeId};
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(PartialEq, Eq, Clone, Default, Debug)]
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
    pub fn instantiate_node(&mut self, entity: Entity) -> NodeId {
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

const TREE_DELIMITER: &'static str = "\u{2022} ";
impl fmt::Display for SceneGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.iter() {
            let node: &Node = node;

            // Okay it's a root node, so we'll handle it
            if node.parent().is_none() {
                writeln!(f, "{}{}", TREE_DELIMITER, node)?;

                if let Some(child) = node.first_child() {
                    for child in Siblings::new(self, child) {
                        pprint_tree(f, child, TREE_DELIMITER.to_string(), self)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn pprint_tree(
    f: &mut fmt::Formatter<'_>,
    node_id: NodeId,
    prefix: String,
    scene_graph: &SceneGraph,
) -> fmt::Result {
    writeln!(
        f,
        "{}{}{}",
        prefix,
        TREE_DELIMITER,
        scene_graph.get(node_id).unwrap()
    )?;
    let prefix = prefix + "   ";

    let node_id_children = node_id.children(scene_graph);
    if node_id_children.count() != 0 {
        for child in node_id.children(scene_graph) {
            pprint_tree(f, child, prefix.to_string(), scene_graph)?;
        }
    }

    Ok(())
}
