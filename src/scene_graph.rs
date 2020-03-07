#[macro_use]
mod relations;

mod graph;
mod graph_id;
mod node;
mod node_error;
mod siblings_range;
mod traverse;

pub use node_error::*;
pub type SceneGraph = graph::Graph<super::Entity>;
pub type Node = node::GraphNode<super::Entity>;
pub type NodeId = graph_id::GraphId<super::Entity>;

pub type SerializedSceneGraph = graph::Graph<uuid::Uuid>;
pub type SerializedNode = node::GraphNode<uuid::Uuid>;
pub type SerializedNodeId = graph_id::GraphId<uuid::Uuid>;

impl SceneGraph {
    pub fn pretty_print(&self, names: &super::ComponentList<super::Name>) {
        self.print_tree(|node| println!("{}", super::Name::get_name_quick(names, node.inner())));
    }
}
