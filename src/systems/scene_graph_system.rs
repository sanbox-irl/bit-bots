use super::{scene_graph::*, ComponentDatabase, ComponentList, Entity, Transform, Vec2};

/// This is code for dumbasses. Do not actually leave it in the game!
pub fn flat_build_headass_code(component_database: &mut ComponentDatabase, scene_graph: &mut SceneGraph) {
    for transform_c in component_database.transforms.iter_mut() {
        let scene_graph_node_id = scene_graph.instantiate_node(transform_c.entity_id());
        let transform: &mut Transform = transform_c.inner_mut();

        transform.scene_graph_node_id = Some(scene_graph_node_id);
    }
}

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

pub fn walk_graph_inspect<T>(component_database: &mut ComponentDatabase, scene_graph: &SceneGraph, mut f: T)
where
    T: FnMut(&Entity, &mut ComponentDatabase, usize, bool) -> bool,
{
    for root_node in scene_graph.iter_roots() {
        walk_node_inspect(root_node, component_database, scene_graph, 0, &mut f);
    }
}

fn walk_node_inspect<T>(
    node: &Node,
    component_database: &mut ComponentDatabase,
    scene_graph: &SceneGraph,
    depth: usize,
    f: &mut T,
) where
    T: FnMut(&Entity, &mut ComponentDatabase, usize, bool) -> bool,
{
    let entity: &Entity = node.inner();
    let has_children = node.first_child().is_some();
    let show_children = f(entity, component_database, depth, has_children);

    if show_children {
        for child in node.children(scene_graph) {
            walk_node_inspect(child, component_database, scene_graph, depth + 1, f);
        }
    }
}
