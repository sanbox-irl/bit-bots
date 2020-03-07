#[macro_use]
mod relations;

mod graph;
mod graph_id;
mod node;
mod node_error;
mod siblings_range;
mod traverse;

pub use node_error::*;

use super::{Entity, SerializationId};

pub type SceneGraph = graph::Graph<Entity>;
pub type Node = node::GraphNode<Entity>;
pub type NodeId = graph_id::GraphId<Entity>;

pub type SerializedSceneGraph = graph::Graph<SerializationId>;
pub type SerializedNode = node::GraphNode<SerializationId>;
pub type SerializedNodeId = graph_id::GraphId<SerializationId>;

impl SceneGraph {
    pub fn pretty_print(&self, names: &super::ComponentList<super::Name>) {
        self.print_tree(|node| println!("{}", super::Name::get_name_quick(names, node.inner())));
    }
}
