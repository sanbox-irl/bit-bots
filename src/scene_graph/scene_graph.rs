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

    /// Returns an iterator only over root nodes, or nodes with no parents who
    /// are not removed.
    pub fn iter_roots(&self) -> impl Iterator<Item = &Node> {
        self.nodes
            .iter()
            .filter(|n| n.is_removed() == false && n.parent().is_none())
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

const TREE_DELIMITER: char = '├';
const TREE_DOWN: char = '└';
const TREE_VERT: char = '|';
impl fmt::Display for SceneGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Scene Graph Root")?;

        let mut top_level_iterator = self.iter_roots().peekable();
        while let Some(node) = top_level_iterator.next() {
            let node: &Node = node;
            let top_level_last = top_level_iterator.peek().is_none();

            writeln!(
                f,
                "{}── {}",
                if top_level_last { TREE_DOWN } else { TREE_DELIMITER },
                node_name = node
            )?;

            let mut iterator = node.children(self).peekable();
            while let Some(child) = iterator.next() {
                let local_last = iterator.peek().is_none();
                pprint_tree(f, child, 3, top_level_last, local_last, self)?;
            }
        }
        Ok(())
    }
}

fn pprint_tree(
    f: &mut fmt::Formatter<'_>,
    node: &Node,
    number_of_spaces: usize,
    main_last: bool,
    local_last: bool,
    scene_graph: &SceneGraph,
) -> fmt::Result {
    // Line and Blank Space...
    write!(f, "{}", if main_last { ' ' } else { TREE_VERT })?;
    for _ in 0..number_of_spaces {
        write!(f, " ")?;
    }

    writeln!(
        f,
        "{}── {}",
        if local_last { TREE_DOWN } else { TREE_DELIMITER },
        node
    )?;

    let mut iterator = node.children(scene_graph).peekable();
    while let Some(child) = iterator.next() {
        let is_last = iterator.peek().is_none();

        pprint_tree(f, child, number_of_spaces + 4, main_last, is_last, scene_graph)?;
    }

    Ok(())
}
