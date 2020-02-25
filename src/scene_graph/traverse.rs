use super::{NodeId, SceneGraph};

// macro_rules! impl_node_iterator {
//     ($name:ident, $next:expr) => {
//         impl<'a> Iterator for $name<'a> {
//             type Item = NodeId;

//             fn next(&mut self) -> Option<NodeId> {
//                 let node = self.node.take()?;
//                 self.node = $next(&self.scene_graph[node]);
//                 Some(node)
//             }
//         }
//     };
// }

#[derive(Clone)]
/// An iterator of references to the ancestors a given node.
pub struct Ancestors<'a> {
    scene_graph: &'a SceneGraph,
    node: Option<NodeId>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].parent;
        Some(node)
    }
}

impl<'a> Ancestors<'a> {
    pub(crate) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
        Self {
            scene_graph,
            node: Some(current),
        }
    }
}

#[derive(Clone)]
/// An iterator of references to the children of a given node.
pub struct Children<'a> {
    scene_graph: &'a SceneGraph,
    node: Option<NodeId>,
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let node = self.node.take()?;
        self.node = self.scene_graph[node].next_sibling;
        Some(node)
    }
}

impl<'a> Children<'a> {
    pub(super) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
        Self {
            scene_graph,
            node: scene_graph[current].first_child,
        }
    }
}

// #[derive(Clone)]
// /// An iterator of references to a given node and its descendants, in tree order.
// pub struct Descendants<'a, T>(Traverse<'a, T>);

// impl<'a, T> Descendants<'a, T> {
//     pub(crate) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
//         Self(Traverse::new(scene_graph, current))
//     }
// }

// impl<'a, T> Iterator for Descendants<'a, T> {
//     type Item = NodeId;

//     fn next(&mut self) -> Option<NodeId> {
//         self.0.find_map(|edge| match edge {
//             NodeEdge::Start(node) => Some(node),
//             NodeEdge::End(_) => None,
//         })
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// /// Indicator if the node is at a start or endpoint of the tree
// pub enum NodeEdge {
//     /// Indicates that start of a node that has children.
//     ///
//     /// Yielded by `Traverse::next()` before the node’s descendants. In HTML or
//     /// XML, this corresponds to an opening tag like `<div>`.
//     Start(NodeId),

//     /// Indicates that end of a node that has children.
//     ///
//     /// Yielded by `Traverse::next()` after the node’s descendants. In HTML or
//     /// XML, this corresponds to a closing tag like `</div>`
//     End(NodeId),
// }

// #[derive(Clone)]
// /// An iterator of references to a given node and its descendants, in tree
// /// order.
// pub struct Traverse<'a, T> {
//     scene_graph: &'a SceneGraph,
//     root: NodeId,
//     next: Option<NodeEdge>,
// }

// impl<'a, T> Traverse<'a, T> {
//     pub(crate) fn new(scene_graph: &'a SceneGraph, current: NodeId) -> Self {
//         Self {
//             scene_graph,
//             root: current,
//             next: Some(NodeEdge::Start(current)),
//         }
//     }

//     /// Calculates the next node.
//     fn next_of_next(&self, next: NodeEdge) -> Option<NodeEdge> {
//         match next {
//             NodeEdge::Start(node) => match self.scene_graph[node].first_child {
//                 Some(first_child) => Some(NodeEdge::Start(first_child)),
//                 None => Some(NodeEdge::End(node)),
//             },
//             NodeEdge::End(node) => {
//                 if node == self.root {
//                     return None;
//                 }
//                 let node = &self.scene_graph[node];
//                 match node.next_sibling {
//                     Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
//                     // `node.parent()` here can only be `None` if the tree has
//                     // been modified during iteration, but silently stoping
//                     // iteration seems a more sensible behavior than panicking.
//                     None => node.parent.map(NodeEdge::End),
//                 }
//             }
//         }
//     }
// }

// impl<'a, T> Iterator for Traverse<'a, T> {
//     type Item = NodeEdge;

//     fn next(&mut self) -> Option<NodeEdge> {
//         let next = self.next.take()?;
//         self.next = self.next_of_next(next);
//         Some(next)
//     }
// }
