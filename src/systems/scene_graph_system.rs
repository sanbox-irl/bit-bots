use super::{
    scene_graph::*, Component, ComponentDatabase, ComponentList, Entity, SerializationMarker, Transform, Vec2,
};

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
        // Update world position
        let last_world_position = transforms
            .get_mut(node.inner())
            .map(|transform| transform.inner_mut().update_world_position(last_world_position))
            .unwrap_or(last_world_position);

        // Propogate this Virus to their Children (you know how we do)
        for child in node.children(scene_graph) {
            update_this_node(child, transforms, last_world_position, scene_graph);
        }
    }

    for root_node in scene_graph.iter_roots() {
        update_this_node(root_node, transforms, Vec2::ZERO, scene_graph);
    }
}

/// This function walks the SceneGraph and converts each reference to a
/// serialized reference. Some important notes here: if any node does NOT
/// have a serialized marker, its direct children will become root nodes. This
/// can have weird behavior if you're not careful!
pub fn create_serialized_graph(
    scene_graph: &SceneGraph,
    serialization_markers: &ComponentList<SerializationMarker>,
) -> SerializedSceneGraph {
    fn walk_serialized_graph(
        node: &Node,
        scene_graph: &SceneGraph,
        parent: Option<NodeId>,
        f: &mut impl FnMut(&Entity, Option<NodeId>) -> Option<NodeId>,
    ) {
        let our_entity: &Entity = node.inner();
        let our_id = f(our_entity, parent);

        for child in node.children(scene_graph) {
            walk_serialized_graph(child, scene_graph, our_id, f);
        }
    }

    let mut serialized_scene_graph = SerializedSceneGraph::new();

    for parent in scene_graph.iter_roots() {
        walk_serialized_graph(parent, scene_graph, None, &mut |entity, parent_id| {
            serialization_markers.get(entity).map(|smc| {
                let id = serialized_scene_graph.instantiate_node(smc.inner().id);

                // Append if we can
                if let Some(parent_id) = parent_id {
                    parent_id.append(id, &mut serialized_scene_graph);
                }

                id
            })
        });
    }

    serialized_scene_graph
}

/// Walks the SceneGraph, giving supporting information. This is for the ImGui
pub fn walk_tree_generically<T>(scene_graph: &SceneGraph, mut f: T)
where
    T: FnMut(&Entity, usize, bool) -> bool,
{
    for root_node in scene_graph.iter_roots() {
        walk_node_generically(root_node, scene_graph, 0, &mut f);
    }
}

fn walk_node_generically<T>(node: &Node, scene_graph: &SceneGraph, depth: usize, f: &mut T)
where
    T: FnMut(&Entity, usize, bool) -> bool,
{
    let entity: &Entity = node.inner();
    let has_children = node.first_child().is_some();
    let show_children = f(entity, depth, has_children);

    if show_children {
        for child in node.children(scene_graph) {
            walk_node_generically(child, scene_graph, depth + 1, f);
        }
    }
}

/// This iterates over the `ComponentDatabase`, finding the `Component<SerializationMarker>`,
/// if it exists, which the `SerializedNode` corresponds to. It then finds and returns
/// the corresponding `Component<Transform>`, if it exists.
pub fn find_transform_from_serialized_node<'a, 'b>(
    component_database: &'a mut ComponentDatabase,
    serialized_node: &'b SerializedNode,
) -> Option<&'a mut Component<Transform>> {
    let sm = &component_database.serialization_markers;
    let tc = &mut component_database.transforms;

    if let Some(entity) = sm.iter().find_map(|smc| {
        if smc.inner().id == *serialized_node.inner() {
            Some(smc.entity_id())
        } else {
            None
        }
    }) {
        return tc.get_mut(&entity);
    }

    None
}
