#[macro_use]
mod relations;

mod graph;
mod node;
mod node_error;
mod node_id;
mod siblings_range;
mod traverse;

pub use node_error::*;
pub use node_id::NodeId;
pub type SceneGraph = graph::Graph<super::Entity>;
pub type Node = node::GraphNode<super::Entity>;

pub type SerializedSceneGraph = graph::Graph<uuid::Uuid>;
pub type SerializedNode = node::GraphNode<uuid::Uuid>;

impl SceneGraph {
    pub fn pretty_print(&self, names: &super::ComponentList<super::Name>) {
        self.print_tree(|node| println!("{}", super::Name::get_name_quick(names, node.inner())));
    }
}
