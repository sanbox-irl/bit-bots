use super::{imgui_component_utils::NameInspectorParameters, *};

#[macro_use]
mod relations;

mod node;
mod node_error;
mod node_id;
mod scene_graph;
mod siblings_range;
mod traverse;

pub use node::*;
pub use node_error::*;
pub use node_id::*;
pub use scene_graph::*;

/// This function walks the SceneGraph, updating each Transform/GraphNode
/// which had previously been marked as "dirty" with a new WorldPosition.
/// Given the Following SceneGraph, for Example:
///
/// ```text
/// Root
///    └-A
///      └--B
/// ```
/// If A's `local_position` has been updated from 0 to 40, and B's `local_position` is
/// at 20, then B's `world_position` will be updated to `60` at the end of
/// this frame.
///
/// This function is the primary responsibility of the scene graph!
pub fn update_transforms_via_scene_graph(
    transforms: &mut ComponentList<Transform>,
    scene_graph: &SceneGraph,
) {
    fn update_this_node(
        node: &Node,
        transforms: &mut ComponentList<Transform>,
        last_world_position: Vec2,
        scene_graph: &SceneGraph,
    ) {
        // Update this Entity's Position if it has A Transform
        // It might not have a Transform if it's just got a GraphNode,
        // which makes it really just a Folder.
        let last_world_position = if let Some(transform) = transforms.get_mut(&node.entity) {
            transform.inner_mut().update_world_position(last_world_position)
        } else {
            last_world_position
        };

        // Propogate this Virus to their Children (you know how we do)
        for child in node.children(scene_graph) {
            update_this_node(child, transforms, last_world_position, scene_graph);
        }
    }

    for root_node in scene_graph.iter_roots() {
        update_this_node(root_node, transforms, Vec2::ZERO, scene_graph);
    }
}

type GraphInspectorLambda<'a> = &'a mut dyn FnMut(
    &Entity,
    &mut ComponentList<Name>,
    &mut ComponentList<SerializationMarker>,
    Option<SerializedEntity>,
    &ComponentList<PrefabMarker>,
    NameInspectorParameters,
) -> bool;

pub fn walk_graph_inspect(
    component_database: &mut super::ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    resources: &ResourcesDatabase,
    scene_graph: &SceneGraph,
    f: GraphInspectorLambda<'_>,
) {
    for root_node in scene_graph.iter_roots() {
        walk_node_inspect(root_node, component_database, singleton_database, resources, 0, f);
    }
}

fn walk_node_inspect(
    node: &Node,
    component_database: &mut ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    resources: &ResourcesDatabase,
    depth: usize,
    f: GraphInspectorLambda<'_>,
) {
    // Unwrap our parts:
    let current_se: Option<SerializedEntity> =
        if let Some(se) = component_database.serialization_markers.get(entity) {
            SerializedEntity::new(
                entity,
                se.inner().id,
                component_database,
                singleton_database,
                resources,
            )
        } else {
            None
        };

    let mut show_children = true;
    let has_children: bool = component_database
        .graph_nodes
        .get(entity)
        .map(|n| {
            if let Some(children) = &n.inner().children {
                children.len() > 0
            } else {
                false
            }
        })
        .unwrap_or_default();

    let has_transform = component_database.transforms.contains(entity);

    if has_transform {
        show_children = f(
            entity,
            &mut component_database.names,
            &mut component_database.serialization_markers,
            current_se,
            &component_database.prefab_markers,
            NameInspectorParameters {
                depth,
                has_children,
                serialization_status: Default::default(),
                prefab_status: Default::default(),
                being_inspected: Default::default(),
            },
        );
    }

    if show_children {
        // We're forced to break Rust borrowing rules here again, because we're a bad bitch.
        let graph_nodes: *const ComponentList<GraphNode> = &component_database.graph_nodes;

        if let Some(this_node) = unsafe { &*graph_nodes }.get(entity) {
            if let Some(children) = &this_node.inner().children {
                for child in children {
                    if let Some(target) = &child.target {
                        walk_node_inspect(
                            target,
                            component_database,
                            singleton_database,
                            resources,
                            depth + 1,
                            f,
                        );
                    }
                }
            }
        }
    }
}
