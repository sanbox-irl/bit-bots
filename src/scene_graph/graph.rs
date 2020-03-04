use super::{node::GraphNode, node_id::NodeId};
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Default, Debug)]
pub struct Graph<T> {
    nodes: Vec<GraphNode<T>>,
}

impl<T> Graph<T> {
    /// Creates a new empty `SceneGraph`.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates a new empty `SceneGraph`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// Create a new node on the scene graph. The new Node ID is not
    /// connected to anything and is therefore a "root" node.
    pub fn instantiate_node(&mut self, data: T) -> NodeId {
        let index = self.nodes.len();
        self.nodes.push(GraphNode::new(data));

        NodeId::new(index)
    }

    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    pub fn get(&self, id: NodeId) -> Option<&GraphNode<T>> {
        self.nodes.get(id.index())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut GraphNode<T>> {
        self.nodes.get_mut(id.index())
    }

    /// Returns a naive iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    pub fn iter(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.nodes.iter()
    }

    /// Returns an iterator only over root nodes, or nodes with no parents who
    /// are not removed.
    pub fn iter_roots(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.nodes
            .iter()
            .filter(|n| n.is_removed() == false && n.parent().is_none())
    }

    /// Walks the SceneGraph, running this function on each member. This really
    /// should be an iterator, but I'm not sure how to convert them without
    /// just experimenting for awhile. I'd like to move to that soon!
    pub fn walk_tree_generically(&self, mut f: impl FnMut(&GraphNode<T>)) {
        for root_node in self.iter_roots() {
            self.walk_node_generically(root_node, &mut f);
        }
    }

    fn walk_node_generically(&self, node: &GraphNode<T>, f: &mut impl FnMut(&GraphNode<T>)) {
        f(node);

        for child in node.children(self) {
            self.walk_node_generically(child, f);
        }
    }
}

impl<T> Index<NodeId> for Graph<T> {
    type Output = GraphNode<T>;

    fn index(&self, index: NodeId) -> &GraphNode<T> {
        &self.nodes[index.index()]
    }
}

impl<T> IndexMut<NodeId> for Graph<T> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.index()]
    }
}

const TREE_DELIMITER: char = '├';
const TREE_DOWN: char = '└';
const TREE_VERT: char = '|';
impl<T: std::fmt::Display> fmt::Display for Graph<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Scene Graph Root")?;

        let mut top_level_iterator = self.iter_roots().peekable();
        while let Some(node) = top_level_iterator.next() {
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

fn pprint_tree<T: std::fmt::Display>(
    f: &mut fmt::Formatter<'_>,
    node: &GraphNode<T>,
    number_of_spaces: usize,
    main_last: bool,
    local_last: bool,
    scene_graph: &Graph<T>,
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
